use crate::config::Config;
use crate::error::{ServerError, ServerResult, UserError};
use crate::server_time_ms;
use crate::state::ServerState;
use crate::websocket::rate_limiter::{RateLimitResult, RateLimiter};
use axum::extract::ws::{Message, WebSocket};
use checkmade_core::data::store::Store;
use checkmade_core::error::DomainError;
use checkmade_core::game::play_move::PlayMove;
use checkmade_core::messages::client::ClientMessage;
use checkmade_core::messages::server::ServerMessage;
use checkmade_core::types::friend_info::{FriendInfo, FriendRequestInfo};
use checkmade_core::types::session_request::CreateSessionRequest;
use checkmade_core::types::session_status::SessionStatus;
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
            ClientMessage::AcceptSessionRequest(session_request_id) => {
                self.handle_accept_session_request(session_request_id.into())
                    .await
            }
            ClientMessage::ActiveSessions => self.handle_active_sessions().await,
            ClientMessage::CreateSessionRequest(session_request) => {
                self.handle_create_session_request(*session_request).await
            }
            ClientMessage::DeclineFriendRequest(target_id) => {
                self.handle_decline_friend_request(target_id.into()).await
            }
            ClientMessage::DeclineSessionRequest(session_request_id) => {
                self.handle_decline_session_request(session_request_id.into())
                    .await
            }
            ClientMessage::Friends => self.handle_friends().await,
            ClientMessage::IncomingFriendRequests => self.handle_incoming_friend_requests().await,
            ClientMessage::IncomingSessionRequests => self.handle_incoming_session_requests().await,
            ClientMessage::OutgoingFriendRequests => self.handle_outgoing_friend_requests().await,
            ClientMessage::OutgoingSessionRequests => self.handle_outgoing_session_requests().await,
            ClientMessage::Ping { client_time } => self.handle_ping(client_time).await,
            ClientMessage::PlayMove { session_id, mv } => {
                self.handle_play_move(session_id.into(), mv).await
            }
            ClientMessage::PrivateUserInfo => self.handle_private_user_info().await,
            ClientMessage::PublicSessionRequests => self.handle_public_session_requests().await,
            ClientMessage::PublicUserInfo(target_id) => {
                self.handle_public_user_info(target_id.into()).await
            }
            ClientMessage::RemoveFriend(target_id) => {
                self.handle_remove_friend(target_id.into()).await
            }
            ClientMessage::RemoveFriendRequest(target_id) => {
                self.handle_remove_friend_request(target_id.into()).await
            }
            ClientMessage::RemoveSessionRequest(session_request_id) => {
                self.handle_remove_session_request(session_request_id.into())
                    .await
            }
            ClientMessage::SendFriendRequest { friend_code } => {
                self.handle_send_friend_request(&friend_code).await
            }
            ClientMessage::Session(session_id) => self.handle_session(session_id.into()).await,
            ClientMessage::SessionHistory => self.handle_session_history().await,
            ClientMessage::SessionRequest(request_id) => {
                self.handle_session_request(request_id.into()).await
            }
            ClientMessage::SubscribeSession(session_id) => {
                self.handle_subscribe_session(session_id.into()).await
            }
            ClientMessage::UnsubscribeSession(session_id) => {
                self.handle_unsubscribe_session(session_id.into()).await
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

        let my_info = self
            .state
            .service
            .friends
            .friend_info(self.user_id, target_id, since)
            .await?;

        let their_info = self
            .state
            .service
            .friends
            .friend_info(target_id, self.user_id, since)
            .await?;

        self.respond(ServerMessage::FriendshipEstablished(my_info));
        self.send_to_user(target_id, ServerMessage::FriendshipEstablished(their_info));

        Ok(())
    }

    async fn handle_accept_session_request(&self, session_request_id: Uuid) -> ServerResult<()> {
        let model = self
            .state
            .data
            .session
            .create(self.user_id, session_request_id)
            .await?;
        let session = self.state.service.session.load_session(model)?;

        self.send_to_user(
            session.white.into(),
            ServerMessage::SessionStart {
                session: session.clone(),
                request_id: session_request_id.into(),
            },
        );
        self.send_to_user(
            session.black.into(),
            ServerMessage::SessionStart {
                session,
                request_id: session_request_id.into(),
            },
        );

        Ok(())
    }

    async fn handle_active_sessions(&self) -> ServerResult<()> {
        let page = self
            .state
            .service
            .session
            .user_page(
                self.user_id,
                Some(SessionStatus::Ongoing),
                self.state.config.core.session_limit,
                0,
            )
            .await?;
        self.respond(ServerMessage::ActiveSessions(page.items));

        Ok(())
    }

    async fn handle_create_session_request(
        &self,
        request: CreateSessionRequest,
    ) -> ServerResult<()> {
        let opponent = request.opponent_id;
        let model = self
            .state
            .data
            .session_request
            .create(self.user_id, request)
            .await?;
        if let Some(opponent) = opponent {
            self.send_to_user(
                opponent.into(),
                ServerMessage::SessionRequestIncoming(model.clone().try_into()?),
            )
        }
        self.respond(ServerMessage::SessionRequestCreateOk(model.try_into()?));
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

    async fn handle_decline_session_request(&self, session_request_id: Uuid) -> ServerResult<()> {
        let requester = self
            .state
            .data
            .session_request
            .decline(self.user_id, session_request_id)
            .await?;
        self.respond(ServerMessage::SessionRequestDeclineOk(
            session_request_id.into(),
        ));
        self.send_to_user(
            requester,
            ServerMessage::FriendRequestDeclinedByPeer(session_request_id.into()),
        );
        Ok(())
    }

    async fn handle_friends(&self) -> ServerResult<()> {
        let page = self
            .state
            .service
            .friends
            .friends_with_stats(self.user_id, self.state.config.core.friend_limit, 0)
            .await?;
        self.respond(ServerMessage::Friends(page.items));
        Ok(())
    }

    async fn handle_incoming_friend_requests(&self) -> ServerResult<()> {
        let friends = self
            .state
            .service
            .friends
            .paginate_received_requests(self.user_id, self.state.config.core.friend_limit)
            .fetch_page(0)
            .await?
            .into_iter()
            .map(|fs| {
                let friend_id = if fs.addressee_id == self.user_id {
                    fs.requester_id
                } else {
                    fs.addressee_id
                };
                FriendRequestInfo {
                    user_id: friend_id.into(),
                    created: fs.created_at.and_utc().timestamp_millis() as u64,
                }
            })
            .collect::<Vec<_>>();
        self.respond(ServerMessage::IncomingFriendRequests(friends));
        Ok(())
    }

    async fn handle_incoming_session_requests(&self) -> ServerResult<()> {
        let session_requests = self
            .state
            .service
            .session
            .incoming_requests_page(
                self.user_id,
                self.state.config.core.session_request_limit,
                0,
            )
            .await?;
        self.respond(ServerMessage::IncomingSessionRequests(
            session_requests.items,
        ));
        Ok(())
    }

    async fn handle_outgoing_friend_requests(&self) -> ServerResult<()> {
        let friends = self
            .state
            .service
            .friends
            .paginate_sent_requests(self.user_id, self.state.config.core.friend_limit)
            .fetch_page(0)
            .await?
            .into_iter()
            .map(|fs| {
                let friend_id = if fs.addressee_id == self.user_id {
                    fs.requester_id
                } else {
                    fs.addressee_id
                };
                FriendRequestInfo {
                    user_id: friend_id.into(),
                    created: fs.created_at.and_utc().timestamp_millis() as u64,
                }
            })
            .collect::<Vec<_>>();
        self.respond(ServerMessage::OutgoingFriendRequests(friends));
        Ok(())
    }

    async fn handle_outgoing_session_requests(&self) -> ServerResult<()> {
        let session_requests = self
            .state
            .service
            .session
            .outgoing_requests_page(
                self.user_id,
                self.state.config.core.session_request_limit,
                0,
            )
            .await?;
        self.respond(ServerMessage::OutgoingSessionRequests(
            session_requests.items,
        ));
        Ok(())
    }

    async fn handle_ping(&self, client_time: u64) -> ServerResult<()> {
        self.respond(ServerMessage::Pong {
            client_time,
            server_time: server_time_ms(),
        });
        Ok(())
    }

    async fn handle_play_move(&self, session_id: Uuid, mv: PlayMove) -> ServerResult<()> {
        let play_time = server_time_ms();
        let (color, model) = self
            .state
            .data
            .session
            .play(self.user_id, session_id, mv.clone(), play_time)
            .await?;

        let update = ServerMessage::SessionUpdate {
            session_id: session_id.into(),
            color,
            mv: mv.clone(),
            at: play_time,
        };

        self.send_to_user(self.user_id, update.clone());

        let opponent_id = if self.user_id == model.white_id {
            model.black_id
        } else {
            model.white_id
        };
        self.send_to_user(opponent_id, update.clone());

        self.state
            .ws
            .broadcast_session(session_id, update, &[self.user_id, opponent_id]);
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

    async fn handle_public_session_requests(&self) -> ServerResult<()> {
        let session_requests = self
            .state
            .service
            .session
            .public_requests_page(100, 0)
            .await?;
        self.respond(ServerMessage::PublicSessionRequests(session_requests.items));
        Ok(())
    }

    async fn handle_public_user_info(&self, target_id: Uuid) -> ServerResult<()> {
        let Some(user) = self.state.data.user.find_by_id(target_id).await? else {
            return Err(UserError::UserNotFound.into());
        };

        let user_online = self.state.ws.is_user_connected(target_id);
        self.respond(ServerMessage::PublicUserInfo(
            self.state.service.user.public_info(&user, user_online),
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

    async fn handle_remove_friend_request(&self, target_id: Uuid) -> ServerResult<()> {
        self.state
            .service
            .friends
            .remove_request(self.user_id, target_id)
            .await?;
        self.respond(ServerMessage::FriendRequestRemoveOk(target_id.into()));
        self.send_to_user(
            target_id,
            ServerMessage::FriendRequestRemovedByPeer(self.user_id.into()),
        );
        Ok(())
    }

    async fn handle_remove_session_request(&self, request_id: Uuid) -> ServerResult<()> {
        let opponent = self
            .state
            .data
            .session_request
            .remove(self.user_id, request_id)
            .await?;
        self.respond(ServerMessage::SessionRequestRemoveOk(request_id.into()));
        if let Some(opponent) = opponent {
            self.send_to_user(
                opponent,
                ServerMessage::SessionRequestRemovedByPeer(request_id.into()),
            );
        }
        Ok(())
    }

    async fn handle_send_friend_request(&self, friend_code: &str) -> ServerResult<()> {
        let fs = self
            .state
            .service
            .friends
            .send_request(self.user_id, friend_code)
            .await?;

        self.respond(ServerMessage::FriendRequestSendOk(FriendRequestInfo {
            user_id: fs.addressee_id.into(),
            created: fs.created_at.and_utc().timestamp_millis() as u64,
        }));

        self.send_to_user(
            fs.addressee_id,
            ServerMessage::FriendRequestIncoming(FriendRequestInfo {
                user_id: self.user_id.into(),
                created: fs.created_at.and_utc().timestamp_millis() as u64,
            }),
        );

        Ok(())
    }

    async fn handle_session(&self, id: Uuid) -> ServerResult<()> {
        let Some(session) = self.state.service.session.load_by_id(id).await? else {
            return Err(ServerError::Core(DomainError::SessionNotFound.into()));
        };
        self.respond(ServerMessage::Session(session));
        Ok(())
    }

    async fn handle_session_history(&self) -> ServerResult<()> {
        // ToDo: Support pagination in frontend and do filtering and stuff
        let page = self
            .state
            .service
            .session
            .user_page(self.user_id, None, 1000, 0)
            .await?;
        self.respond(ServerMessage::Sessions(page.items));
        Ok(())
    }

    async fn handle_session_request(&self, id: Uuid) -> ServerResult<()> {
        let Some(request) = self.state.service.session.load_request_by_id(id).await? else {
            return Err(ServerError::Core(
                DomainError::SessionRequestNotFound.into(),
            ));
        };
        self.respond(ServerMessage::SessionRequest(request));
        Ok(())
    }

    async fn handle_subscribe_session(&self, session_id: Uuid) -> ServerResult<()> {
        self.state.ws.subscribe_to_session(self.id, session_id);
        Ok(())
    }

    async fn handle_unsubscribe_session(&self, session_id: Uuid) -> ServerResult<()> {
        self.state.ws.unsubscribe_from_session(self.id, session_id);
        Ok(())
    }
}
