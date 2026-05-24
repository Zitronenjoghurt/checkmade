use crate::client_time_ms;
use crate::event::{AppEvent, DisconnectedEvent, ErrorEvent, InfoEvent, ReconnectedEvent};
use crate::i18n::Translatable;
use crate::utils::fmt::fmt_duration;
use checkmade_core::lingo::Lingo::AttemptingReconnect;
use checkmade_core::messages::client::ClientMessage;
use checkmade_core::messages::server::ServerMessage;
use checkmade_core::types::session_request::{CreateSessionRequest, SessionRequestId};
use checkmade_core::types::user_id::UserId;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};

pub mod cache;
pub mod fetchable;

#[derive(Default)]
pub enum WsState {
    #[default]
    Idle,
    Connecting,
    Connected,
    Error(String),
}

pub struct Ws {
    url: Option<String>,
    state: WsState,
    sender: Option<WsSender>,
    receiver: Option<WsReceiver>,
    incoming: Vec<ServerMessage>,
    was_connected: bool,
    reconnect_at: Option<u64>,
    reconnect_attempt: u32,
}

impl Default for Ws {
    fn default() -> Self {
        Self {
            url: None,
            state: WsState::Idle,
            sender: None,
            receiver: None,
            incoming: Vec::new(),
            was_connected: false,
            reconnect_at: None,
            reconnect_attempt: 0,
        }
    }
}

impl Ws {
    pub fn connect(&mut self, url: impl Into<String>) {
        let url = url.into();
        self.url = Some(url.clone());

        if matches!(self.state, WsState::Idle | WsState::Error(_)) {
            self.state = WsState::Connecting;
            match ewebsock::connect(url, ewebsock::Options::default()) {
                Ok((sender, receiver)) => {
                    self.sender = Some(sender);
                    self.receiver = Some(receiver);
                }
                Err(err) => {
                    self.state = WsState::Error(err.to_string());
                }
            }
        }
    }

    pub fn disconnect(&mut self) {
        self.sender = None;
        self.receiver = None;
        self.state = WsState::Idle;
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, WsState::Connected)
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        let Some(receiver) = &self.receiver else {
            return;
        };

        while let Some(event) = receiver.try_recv() {
            match event {
                WsEvent::Opened => {
                    self.state = WsState::Connected;
                }
                WsEvent::Message(message) => {
                    if let WsMessage::Binary(bytes) = message {
                        match ServerMessage::from_bytes(&bytes) {
                            Ok(msg) => self.incoming.push(msg),
                            Err(err) => {
                                ErrorEvent(format!("Failed to parse message: {}", err)).send(ctx);
                            }
                        }
                    }
                }
                WsEvent::Error(err) => {
                    self.state = WsState::Error(err.to_string());
                }
                WsEvent::Closed => {
                    self.state = WsState::Error("Connection closed".to_string());
                }
            }
        }

        let connected = self.is_connected();
        if self.was_connected && !connected {
            DisconnectedEvent.send(ctx);
            self.schedule_reconnect(ctx);
        }
        if !self.was_connected && connected {
            ReconnectedEvent.send(ctx);
            self.reconnect_at = None;
            self.reconnect_attempt = 0;
        }
        self.was_connected = connected;

        if let (Some(url), Some(at)) = (self.url.clone(), self.reconnect_at)
            && !connected
            && client_time_ms() >= at
        {
            self.schedule_reconnect(ctx);
            self.connect(url);
        }
    }

    pub fn drain_incoming(&mut self) -> std::vec::Drain<'_, ServerMessage> {
        self.incoming.drain(..)
    }

    pub fn send(&mut self, msg: ClientMessage) {
        if let Some(sender) = &mut self.sender {
            sender.send(WsMessage::Binary(msg.as_bytes()));
        }
    }

    fn schedule_reconnect(&mut self, ctx: &egui::Context) {
        let delay = (3000 * 2u64.pow(self.reconnect_attempt.min(3))).min(60_000);
        self.reconnect_at = Some(client_time_ms() + delay);
        self.reconnect_attempt += 1;
        let duration = web_time::Duration::from_millis(delay);
        InfoEvent(format!(
            "{}... ({})",
            AttemptingReconnect.t(),
            fmt_duration(duration)
        ))
        .send(ctx);
    }
}

// Message helpers
impl Ws {
    pub fn ping(&mut self) {
        self.send(ClientMessage::Ping {
            client_time: client_time_ms(),
        });
    }

    pub fn request_active_sessions(&mut self) {
        self.send(ClientMessage::ActiveSessions);
    }

    pub fn request_friends(&mut self) {
        self.send(ClientMessage::Friends);
    }

    pub fn request_incoming_friend_requests(&mut self) {
        self.send(ClientMessage::IncomingFriendRequests);
    }

    pub fn request_incoming_session_requests(&mut self) {
        self.send(ClientMessage::IncomingSessionRequests);
    }

    pub fn request_outgoing_friend_requests(&mut self) {
        self.send(ClientMessage::OutgoingFriendRequests);
    }

    pub fn request_outgoing_session_requests(&mut self) {
        self.send(ClientMessage::OutgoingSessionRequests);
    }

    pub fn request_public_session_requests(&mut self) {
        self.send(ClientMessage::PublicSessionRequests);
    }

    pub fn request_private_user_info(&mut self) {
        self.send(ClientMessage::PrivateUserInfo);
    }

    pub fn request_public_user_info(&mut self, id: UserId) {
        self.send(ClientMessage::PublicUserInfo(id))
    }

    pub fn accept_friend_request(&mut self, id: UserId) {
        self.send(ClientMessage::AcceptFriendRequest(id));
    }

    pub fn decline_friend_request(&mut self, id: UserId) {
        self.send(ClientMessage::DeclineFriendRequest(id));
    }

    pub fn send_friend_request(&mut self, friend_code: String) {
        self.send(ClientMessage::SendFriendRequest { friend_code });
    }

    pub fn remove_friend(&mut self, id: UserId) {
        self.send(ClientMessage::RemoveFriend(id));
    }

    pub fn remove_friend_request(&mut self, id: UserId) {
        self.send(ClientMessage::RemoveFriendRequest(id));
    }

    pub fn remove_session_request(&mut self, id: SessionRequestId) {
        self.send(ClientMessage::RemoveSessionRequest(id));
    }

    pub fn create_session_request(&mut self, request: CreateSessionRequest) {
        self.send(ClientMessage::CreateSessionRequest(Box::new(request)));
    }

    pub fn accept_session_request(&mut self, request_id: SessionRequestId) {
        self.send(ClientMessage::AcceptSessionRequest(request_id));
    }

    pub fn decline_session_request(&mut self, request_id: SessionRequestId) {
        self.send(ClientMessage::DeclineSessionRequest(request_id));
    }
}
