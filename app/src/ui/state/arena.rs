use crate::store::Store;
use crate::ws::Ws;
use checkmade_core::game::play_move::PlayMove;
use checkmade_core::game::visuals::BoardVisuals;
use checkmade_core::giga_chess::prelude::action::SessionAction;
use checkmade_core::giga_chess::prelude::{Color, Game, Square};
use checkmade_core::messages::client::ClientMessage;
use checkmade_core::types::session_id::SessionId;
use checkmade_core::types::user_id::UserId;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ArenaState {
    #[serde(skip, default)]
    pub subscribed_session: Option<SessionId>,
    pub perspective: Color,
    pub source: ArenaSource,
}

impl Default for ArenaState {
    fn default() -> Self {
        Self {
            subscribed_session: None,
            perspective: Color::White,
            source: ArenaSource::default(),
        }
    }
}

impl ArenaState {
    pub fn visuals(&self, user_id: UserId, store: &Store) -> Option<BoardVisuals> {
        self.source.visuals(user_id, self.perspective, store)
    }

    pub fn handle_move(&mut self, from: Square, to: Square, ws: &mut Ws) {
        self.source.handle_move(from, to, ws);
    }

    pub fn has_session(&self, id: SessionId) -> bool {
        self.source.has_session(id)
    }

    pub fn session_id(&self) -> Option<SessionId> {
        self.source.session_id()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ArenaSource {
    Active(SessionId),
    Sandbox(Game),
}

impl Default for ArenaSource {
    fn default() -> Self {
        Self::Sandbox(Game::default())
    }
}

impl ArenaSource {
    pub fn visuals(
        &self,
        user_id: UserId,
        perspective: Color,
        store: &Store,
    ) -> Option<BoardVisuals> {
        match self {
            Self::Active(id) => {
                let Some(session) = store.active_sessions.get_entry(id) else {
                    return None;
                };
                Some(session.visuals(user_id, perspective))
            }
            Self::Sandbox(game) => Some(BoardVisuals::from_game(perspective, game)),
        }
    }

    pub fn handle_move(&mut self, from: Square, to: Square, ws: &mut Ws) {
        match self {
            ArenaSource::Sandbox(game) => {
                if let Some(mv) = game.find_move(from, to, None) {
                    game.play_move(mv).unwrap();
                }
            }
            ArenaSource::Active(id) => {
                ws.send(ClientMessage::PlayMove {
                    session_id: *id,
                    mv: PlayMove::Normal(SessionAction::MoveFromTo {
                        from,
                        to,
                        promotion: None,
                    }),
                });
            }
        }
    }

    pub fn has_session(&self, id: SessionId) -> bool {
        match self {
            ArenaSource::Active(session_id) => id == *session_id,
            ArenaSource::Sandbox(_) => false,
        }
    }

    pub fn session_id(&self) -> Option<SessionId> {
        match self {
            Self::Active(id) => Some(*id),
            Self::Sandbox(_) => None,
        }
    }
}
