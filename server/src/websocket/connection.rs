use crate::config::Config;
use crate::error::{ServerError, ServerResult, UserError};
use crate::server_time_ms;
use crate::state::ServerState;
use crate::websocket::rate_limiter::{RateLimitResult, RateLimiter};
use axum::extract::ws::{Message, WebSocket};
use checkmade_core::data::store::Store;
use checkmade_core::messages::client::ClientMessage;
use checkmade_core::messages::server::ServerMessage;
use checkmade_core::types::friend_info::FriendInfo;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use metrics::{counter, gauge};
use std::ops::ControlFlow;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info};
use uuid::Uuid;

pub type ConnectionId = Uuid;

const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(60);

pub struct WebsocketConnection {
    id: ConnectionId,
    user_id: Uuid,
    state: ServerState,
    rate_limiter: RateLimiter,
}

impl WebsocketConnection {
    pub fn new(config: &Config, id: ConnectionId, user_id: Uuid, state: ServerState) -> Self {
        Self {
            id,
            user_id,
            state,
            rate_limiter: RateLimiter::new(config),
        }
    }

    pub async fn handle_receive(mut self, mut ws_receive: SplitStream<WebSocket>) {
        gauge!("ws.active_connections").increment(1.0);

        loop {
            let msg = match timeout(HEARTBEAT_TIMEOUT, ws_receive.next()).await {
                Ok(Some(Ok(message))) => message,
                Ok(Some(Err(err))) => {
                    error!("[{}] WebSocket error: {err}", self.user_id);
                    counter!("ws.disconnect_total", "reason" => "error").increment(1);
                    break;
                }
                Ok(None) => {
                    info!("[{}] WebSocket stream closed", self.user_id);
                    counter!("ws.disconnect_total", "reason" => "stream_closed").increment(1);
                    break;
                }
                Err(_) => {
                    info!(
                        "[{}] Connection timed out (idle for {HEARTBEAT_TIMEOUT:?})",
                        self.user_id
                    );
                    counter!("ws.disconnect_total", "reason" => "heartbeat_timeout").increment(1);
                    break;
                }
            };

            match msg {
                Message::Binary(data) if self.handle_binary(&data).await.is_break() => break,
                Message::Close(reason) => {
                    info!("[{}] Client closed connection: {reason:?}", self.user_id);
                    counter!("ws.disconnect_total", "reason" => "client_close").increment(1);
                    break;
                }
                _ => {}
            }
        }

        gauge!("ws.active_connections").decrement(1.0);
    }

    async fn handle_binary(&mut self, data: &[u8]) -> ControlFlow<()> {
        metrics::histogram!("ws.inbound_size_bytes").record(data.len() as f64);

        let message = match ClientMessage::from_bytes(data) {
            Ok(msg) => msg,
            Err(e) => {
                error!("[{}] Failed to decode message: {e}", self.user_id);
                return ControlFlow::Continue(());
            }
        };

        match self.rate_limiter.check(&self.state.config, &message) {
            RateLimitResult::Allow => {}
            RateLimitResult::Drop => {
                self.respond(ServerMessage::Error(
                    "Too many requests. Please wait a moment.".into(),
                ));
                counter!("ws.rate_limit_total", "action" => "drop").increment(1);
                return ControlFlow::Continue(());
            }
            RateLimitResult::Warn => {
                self.respond(ServerMessage::Error(
                    "You are being rate limited. Continued excessive requests may result in a disconnect.".into(),
                ));
                counter!("ws.rate_limit_total", "action" => "warn").increment(1);
                return ControlFlow::Continue(());
            }
            RateLimitResult::Disconnect => {
                let infractions = self
                    .state
                    .service
                    .user
                    .log_rate_limit_infraction(self.user_id)
                    .await
                    .unwrap_or_default();
                self.respond(ServerMessage::Error(
                    "Connection closed due to excessive requests. Continued abuse will lead to a permanent ban.".into(),
                ));
                info!(
                    "[{}] Disconnected for rate limit abuse, {infractions} infractions.",
                    self.user_id
                );
                counter!("ws.disconnect_total", "reason" => "rate_limit").increment(1);
                counter!("ws.rate_limit_total", "action" => "disconnect").increment(1);
                return ControlFlow::Break(());
            }
        }

        if let Err(err) = self.handle_client_message(message).await {
            if err.is_user_error() {
                self.respond(ServerMessage::Error(err.message()));
            } else {
                error!(
                    "[{}] An error occurred on message handling: {err}",
                    self.user_id
                );
            }
        }

        ControlFlow::Continue(())
    }

    fn respond(&self, message: ServerMessage) {
        self.state.ws.send_to_connection(self.id, message);
    }

    fn send_to_user(&self, user_id: Uuid, message: ServerMessage) {
        self.state.ws.send_to_user(user_id, message);
    }

    async fn handle_client_message(&self, message: ClientMessage) -> ServerResult<()> {
        let msg_type = message.name();
        counter!("ws.inbound_total", "type" => msg_type).increment(1);

        let start = std::time::Instant::now();

        let result = match message {
            ClientMessage::AcceptFriendRequest(target_id) => {
                self.handle_accept_friend_request(target_id.into()).await
            }
            ClientMessage::DeclineFriendRequest(target_id) => {
                self.handle_decline_friend_request(target_id.into()).await
            }
            ClientMessage::Friends => self.handle_friends().await,
            ClientMessage::IncomingFriendRequests => self.handle_incoming_friend_requests().await,
            ClientMessage::OutgoingFriendRequests => self.handle_outgoing_friend_requests().await,
            ClientMessage::Ping { client_time } => self.handle_ping(client_time).await,
            ClientMessage::PrivateUserInfo => self.handle_private_user_info().await,
            ClientMessage::PublicUserInfo(target_id) => {
                self.handle_public_user_info(target_id.into()).await
            }
            ClientMessage::RemoveFriend(target_id) => {
                self.handle_remove_friend(target_id.into()).await
            }
            ClientMessage::SendFriendRequest { friend_code } => {
                self.handle_send_friend_request(&friend_code).await
            }
        };

        let elapsed = start.elapsed().as_secs_f64();
        metrics::histogram!("ws.inbound_duration_secs", "type" => msg_type).record(elapsed);

        result
    }
}

// Message handling
impl WebsocketConnection {
    async fn handle_accept_friend_request(&self, target_id: Uuid) -> ServerResult<()> {
        let fs = self
            .state
            .service
            .friends
            .accept_request(self.user_id, target_id)
            .await?;

        let since = fs.created_at.and_utc().timestamp_millis() as u64;
        self.respond(ServerMessage::FriendshipEstablished(FriendInfo {
            user_id: target_id.into(),
            since,
        }));
        self.send_to_user(
            target_id,
            ServerMessage::FriendshipEstablished(FriendInfo {
                user_id: self.user_id.into(),
                since,
            }),
        );

        Ok(())
    }

    async fn handle_decline_friend_request(&self, target_id: Uuid) -> ServerResult<()> {
        self.state
            .service
            .friends
            .reject_request(self.user_id, target_id)
            .await?;

        self.respond(ServerMessage::FriendRequestDeclineOk(target_id.into()));
        self.send_to_user(
            target_id,
            ServerMessage::FriendRequestDeclinedByPeer(self.user_id.into()),
        );

        Ok(())
    }

    async fn handle_friends(&self) -> ServerResult<()> {
        let friends = self
            .state
            .service
            .friends
            .paginate_friends(self.user_id, self.state.config.friend_limit as u64)
            .fetch_page(1)
            .await?
            .into_iter()
            .map(|fs| {
                let friend_id = if fs.addressee_id == self.user_id {
                    fs.requester_id
                } else {
                    fs.addressee_id
                };
                FriendInfo {
                    user_id: friend_id.into(),
                    since: fs.created_at.and_utc().timestamp_millis() as u64,
                }
            })
            .collect::<Vec<_>>();
        self.respond(ServerMessage::Friends(friends));

        Ok(())
    }

    async fn handle_incoming_friend_requests(&self) -> ServerResult<()> {
        let friends = self
            .state
            .service
            .friends
            .paginate_received_requests(self.user_id, self.state.config.friend_limit as u64)
            .fetch_page(1)
            .await?
            .into_iter()
            .map(|fs| {
                let friend_id = if fs.addressee_id == self.user_id {
                    fs.requester_id
                } else {
                    fs.addressee_id
                };
                FriendInfo {
                    user_id: friend_id.into(),
                    since: fs.created_at.and_utc().timestamp_millis() as u64,
                }
            })
            .collect::<Vec<_>>();
        self.respond(ServerMessage::IncomingFriendRequests(friends));
        Ok(())
    }

    async fn handle_outgoing_friend_requests(&self) -> ServerResult<()> {
        let friends = self
            .state
            .service
            .friends
            .paginate_sent_requests(self.user_id, self.state.config.friend_limit as u64)
            .fetch_page(1)
            .await?
            .into_iter()
            .map(|fs| {
                let friend_id = if fs.addressee_id == self.user_id {
                    fs.requester_id
                } else {
                    fs.addressee_id
                };
                FriendInfo {
                    user_id: friend_id.into(),
                    since: fs.created_at.and_utc().timestamp_millis() as u64,
                }
            })
            .collect::<Vec<_>>();
        self.respond(ServerMessage::OutgoingFriendRequests(friends));
        Ok(())
    }

    async fn handle_ping(&self, client_time: u64) -> ServerResult<()> {
        self.respond(ServerMessage::Pong {
            client_time,
            server_time: server_time_ms(),
        });
        Ok(())
    }

    async fn handle_private_user_info(&self) -> ServerResult<()> {
        let Some(user) = self.state.data.user.find_by_id(self.user_id).await? else {
            return Err(UserError::Unauthorized.into());
        };

        self.respond(ServerMessage::PrivateUserInfo(
            self.state.service.user.private_info(&user),
        ));

        Ok(())
    }

    async fn handle_public_user_info(&self, target_id: Uuid) -> ServerResult<()> {
        let Some(user) = self.state.data.user.find_by_id(target_id).await? else {
            return Err(UserError::UserNotFound.into());
        };

        self.respond(ServerMessage::PublicUserInfo(
            self.state.service.user.public_info(&user),
        ));

        Ok(())
    }

    async fn handle_remove_friend(&self, target_id: Uuid) -> ServerResult<()> {
        self.state
            .service
            .friends
            .remove_friend(self.user_id, target_id)
            .await?;

        self.respond(ServerMessage::FriendRemoveOk(target_id.into()));
        self.send_to_user(
            target_id,
            ServerMessage::FriendshipRemovedByPeer(self.user_id.into()),
        );

        Ok(())
    }

    async fn handle_send_friend_request(&self, friend_code: &str) -> ServerResult<()> {
        let fs = self
            .state
            .service
            .friends
            .send_request(self.user_id, friend_code)
            .await?;

        self.respond(ServerMessage::FriendRequestSendOk(FriendInfo {
            user_id: fs.addressee_id.into(),
            since: fs.created_at.and_utc().timestamp_millis() as u64,
        }));

        self.send_to_user(
            fs.addressee_id,
            ServerMessage::FriendRequestIncoming(FriendInfo {
                user_id: self.user_id.into(),
                since: fs.created_at.and_utc().timestamp_millis() as u64,
            }),
        );

        Ok(())
    }
}
