use crate::websocket::connection::ConnectionId;
use checkmade_core::data::service::Services;
use checkmade_core::data::store::Store;
use checkmade_core::data::Data;
use checkmade_core::game::play_move::PlayMove;
use checkmade_core::game::play_session::PlaySession;
use checkmade_core::messages::server::ServerMessage;
use checkmade_core::types::session_id::SessionId;
use checkmade_core::types::user_info::PublicUserInfo;
use dashmap::DashMap;
use futures_util::TryStreamExt;
use metrics::{counter, gauge};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

mod connection;
pub mod handler;
mod rate_limiter;
mod subscriptions;

pub struct Websocket {
    connections: DashMap<ConnectionId, (Uuid, mpsc::Sender<ServerMessage>)>,
    users: DashMap<Uuid, HashSet<ConnectionId>>,
    data: Arc<Data>,
    service: Arc<Services>,
    subscriptions: subscriptions::Subscriptions,
}

impl Websocket {
    pub fn new(data: &Arc<Data>, service: &Arc<Services>) -> Self {
        Self {
            connections: DashMap::new(),
            users: DashMap::new(),
            data: Arc::clone(data),
            service: Arc::clone(service),
            subscriptions: subscriptions::Subscriptions::new(),
        }
    }

    pub fn register(
        self: &Arc<Self>,
        user_id: Uuid,
    ) -> (ConnectionId, mpsc::Receiver<ServerMessage>) {
        let connection_id = ConnectionId::new_v4();
        let (tx, rx) = mpsc::channel(100);
        self.connections.insert(connection_id, (user_id, tx));

        let (count, is_first) = {
            let mut entry = self.users.entry(user_id).or_default();
            let is_first = entry.is_empty();
            entry.insert(connection_id);
            (entry.len(), is_first)
        };

        info!("User '{user_id}' connected ({connection_id}). Active connections for user: {count}");
        gauge!("ws.unique_users").set(self.users.len() as f64);

        if is_first {
            let this = Arc::clone(self);
            tokio::spawn(async move {
                this.on_user_online(user_id).await;
            });
        }

        (connection_id, rx)
    }

    pub fn unregister(self: &Arc<Self>, connection_id: ConnectionId) {
        let Some((_, (user_id, _))) = self.connections.remove(&connection_id) else {
            return;
        };

        info!("Connection '{connection_id}' of user '{user_id}' closed.");

        let user_empty = {
            let Some(mut entry) = self.users.get_mut(&user_id) else {
                return;
            };
            self.subscriptions.unsubscribe_all(connection_id);
            entry.remove(&connection_id);
            entry.is_empty()
        };

        if user_empty {
            self.users.remove(&user_id);
            info!("All connections closed for '{}'.", user_id);
            gauge!("ws.unique_users").set(self.users.len() as f64);

            let this = Arc::clone(self);
            tokio::spawn(async move {
                this.on_user_offline(user_id).await;
            });
        }
    }

    pub fn send_to_connection(&self, connection_id: ConnectionId, message: ServerMessage) {
        let Some(conn) = self.connections.get(&connection_id) else {
            return;
        };
        if conn.1.try_send(message).is_err() {
            counter!("ws.outbound_dropped").increment(1);
        }
    }

    pub fn send_to_user(&self, user_id: Uuid, message: ServerMessage) {
        let Some(connection_ids) = self.users.get(&user_id) else {
            return;
        };

        if connection_ids.len() == 1 {
            self.send_to_connection(*connection_ids.iter().next().unwrap(), message);
        } else {
            for connection_id in connection_ids.iter() {
                self.send_to_connection(*connection_id, message.clone());
            }
        }
    }

    pub fn is_user_connected(&self, user_id: Uuid) -> bool {
        self.users.contains_key(&user_id)
    }

    pub fn user_connection_count(&self, user_id: Uuid) -> usize {
        self.users.get(&user_id).map(|s| s.len()).unwrap_or(0)
    }

    pub fn connection_user_id(&self, connection_id: ConnectionId) -> Option<Uuid> {
        self.connections.get(&connection_id).map(|c| c.0)
    }

    pub fn subscribe_to_session(&self, connection_id: ConnectionId, session_id: Uuid) {
        self.subscriptions
            .subscribe(connection_id, session_id.into());
    }

    pub fn unsubscribe_from_session(&self, connection_id: ConnectionId, session_id: Uuid) {
        self.subscriptions
            .unsubscribe(connection_id, session_id.into());
    }

    pub fn broadcast_session(&self, session_id: Uuid, mv: PlayMove) {
        self.subscriptions
            .with_subscribers(&session_id, |subscribers| {
                for conn in subscribers {
                    self.send_to_connection(
                        *conn,
                        ServerMessage::SessionUpdate {
                            session_id: session_id.into(),
                            mv: mv.clone(),
                        },
                    );
                }
            });
    }
}

// Event handling
impl Websocket {
    pub async fn on_user_online(&self, id: Uuid) {
        let Ok(Some(user)) = self.data.user.find_by_id(id).await else {
            return;
        };
        let info = self.service.user.public_info(&user, true);
        self.on_friend_user_info_update(info).await;
    }

    pub async fn on_user_offline(&self, id: Uuid) {
        let Ok(Some(user)) = self.data.user.find_by_id(id).await else {
            return;
        };
        let info = self.service.user.public_info(&user, false);
        self.on_friend_user_info_update(info).await;
    }

    pub async fn on_friend_user_info_update(&self, info: PublicUserInfo) {
        let Ok(mut friend_id_stream) = self.data.friends.stream_friend_ids_of(info.id.into()).await
        else {
            return;
        };
        while let Some(id) = friend_id_stream.try_next().await.unwrap_or(None) {
            self.send_to_user(id, ServerMessage::PublicUserInfo(info.clone()));
        }
    }
}
