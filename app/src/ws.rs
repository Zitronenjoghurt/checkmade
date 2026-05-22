use crate::client_time_ms;
use checkmade_core::messages::client::ClientMessage;
use checkmade_core::messages::server::ServerMessage;
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
    state: WsState,
    sender: Option<WsSender>,
    receiver: Option<WsReceiver>,
    incoming: Vec<ServerMessage>,
}

impl Default for Ws {
    fn default() -> Self {
        Self {
            state: WsState::Idle,
            sender: None,
            receiver: None,
            incoming: Vec::new(),
        }
    }
}

impl Ws {
    pub fn connect(&mut self, url: impl Into<String>) {
        if matches!(self.state, WsState::Idle | WsState::Error(_)) {
            self.state = WsState::Connecting;
            match ewebsock::connect(url.into(), ewebsock::Options::default()) {
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

    pub fn update(&mut self, toasts: &mut egui_notify::Toasts) {
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
                                toasts.error(err.to_string());
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
    }

    pub fn drain_incoming(&mut self) -> std::vec::Drain<'_, ServerMessage> {
        self.incoming.drain(..)
    }

    pub fn send(&mut self, msg: ClientMessage) {
        if let Some(sender) = &mut self.sender {
            sender.send(WsMessage::Binary(msg.as_bytes()));
        }
    }
}

// Message helpers
impl Ws {
    pub fn ping(&mut self) {
        self.send(ClientMessage::Ping {
            client_time: client_time_ms(),
        });
    }

    pub fn request_friends(&mut self) {
        self.send(ClientMessage::Friends);
    }

    pub fn request_incoming_friend_requests(&mut self) {
        self.send(ClientMessage::IncomingFriendRequests);
    }

    pub fn request_outgoing_friend_requests(&mut self) {
        self.send(ClientMessage::OutgoingFriendRequests);
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
}
