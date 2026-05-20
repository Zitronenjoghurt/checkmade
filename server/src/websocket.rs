use crate::websocket::connection::ConnectionId;
use checkmade_core::messages::server::ServerMessage;
use dashmap::DashMap;
use metrics::{counter, gauge};
use std::collections::HashSet;
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

mod connection;
pub mod handler;
mod rate_limiter;

#[derive(Default)]
pub struct Websocket {
    connections: DashMap<ConnectionId, (Uuid, mpsc::Sender<ServerMessage>)>,
    users: DashMap<Uuid, HashSet<ConnectionId>>,
}

impl Websocket {
    pub fn register(&self, user_id: Uuid) -> (ConnectionId, mpsc::Receiver<ServerMessage>) {
        let connection_id = ConnectionId::new_v4();
        let (tx, rx) = mpsc::channel(100);
        self.connections.insert(connection_id, (user_id, tx));

        let count = {
            let mut entry = self.users.entry(user_id).or_default();
            entry.insert(connection_id);
            entry.len()
        };

        info!("User '{user_id}' connected ({connection_id}). Active connections for user: {count}");
        gauge!("ws.unique_users").set(self.users.len() as f64);
        (connection_id, rx)
    }

    pub fn unregister(&self, connection_id: ConnectionId) {
        let Some((_, (user_id, _))) = self.connections.remove(&connection_id) else {
            return;
        };

        info!("Connection '{connection_id}' of user '{user_id}' closed.");

        let user_empty = {
            let Some(mut entry) = self.users.get_mut(&user_id) else {
                return;
            };
            entry.remove(&connection_id);
            entry.is_empty()
        };

        if user_empty {
            self.users.remove(&user_id);
            info!("All connections closed for '{}'.", user_id);
            gauge!("ws.unique_users").set(self.users.len() as f64);
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
}
