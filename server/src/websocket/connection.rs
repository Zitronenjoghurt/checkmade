use crate::config::Config;
use crate::error::{ServerError, ServerResult};
use crate::server_time_ms;
use crate::state::ServerState;
use crate::websocket::rate_limiter::{RateLimitResult, RateLimiter};
use axum::extract::ws::{Message, WebSocket};
use checkmade_core::data::store::Store;
use checkmade_core::messages::client::ClientMessage;
use checkmade_core::messages::server::ServerMessage;
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

        match message {
            ClientMessage::Ping { client_time } => self.handle_ping(client_time).await?,
            ClientMessage::UserInfo => self.handle_user_info().await?,
        };

        let elapsed = start.elapsed().as_secs_f64();
        metrics::histogram!("ws.inbound_duration_secs", "type" => msg_type).record(elapsed);

        Ok(())
    }
}

// Message handling
impl WebsocketConnection {
    async fn handle_ping(&self, client_time: u64) -> ServerResult<()> {
        self.respond(ServerMessage::Pong {
            client_time,
            server_time: server_time_ms(),
        });
        Ok(())
    }

    async fn handle_user_info(&self) -> ServerResult<()> {
        let Some(user) = self.state.data.user.find_by_id(self.user_id).await? else {
            return Err(ServerError::Unauthorized);
        };

        self.respond(ServerMessage::UserInfo(
            self.state.service.user.private_info(&user),
        ));

        Ok(())
    }
}
