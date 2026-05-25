use crate::store::Store;
use crate::ws::Ws;
use checkmade_core::game::play_move::PlayMove;
use checkmade_core::game::visuals::BoardVisuals;
use checkmade_core::giga_chess::prelude::action::SessionAction;
use checkmade_core::giga_chess::prelude::{Color, Game, Piece, Square};
use checkmade_core::messages::client::ClientMessage;
use checkmade_core::types::session_id::SessionId;
use checkmade_core::types::user_id::UserId;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ArenaState {
    #[serde(skip, default)]
    pub subscribed_session: Option<SessionId>,
    pub perspective: Color,
    pub source: ArenaSource,
    #[serde(skip, default)]
    pub pending_promotion: Option<(Square, Square)>,
}

impl Default for ArenaState {
    fn default() -> Self {
        Self {
            subscribed_session: None,
            perspective: Color::White,
            source: ArenaSource::default(),
            pending_promotion: None,
        }
    }
}

impl ArenaState {
    pub fn visuals(&self, user_id: UserId, store: &Store) -> Option<ArenaVisuals> {
        self.source.visuals(user_id, self.perspective, store)
    }

    pub fn handle_move(&mut self, from: Square, to: Square, promotion: Option<Piece>, ws: &mut Ws) {
        self.source.handle_move(from, to, promotion, ws);
    }

    pub fn has_session(&self, id: SessionId) -> bool {
        self.source.has_session(id)
    }

    pub fn session_id(&self) -> Option<SessionId> {
        self.source.session_id()
    }

    pub fn movement(&self, store: &Store, user_id: UserId) -> (bool, Option<Color>) {
        self.source.movement(store, user_id)
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
    ) -> Option<ArenaVisuals> {
        match self {
            Self::Active(id) => {
                let session = store.active_sessions.get_entry(id)?;
                let board = session.visuals(user_id, perspective);
                let (bottom_player, top_player) = if board.perspective == Color::White {
                    ((session.white, Color::White), (session.black, Color::Black))
                } else {
                    ((session.black, Color::Black), (session.white, Color::White))
                };
                Some(ArenaVisuals {
                    board,
                    top_player: Some(top_player),
                    bottom_player: Some(bottom_player),
                })
            }
            Self::Sandbox(game) => Some(ArenaVisuals {
                board: BoardVisuals::from_game(perspective, game),
                top_player: None,
                bottom_player: None,
            }),
        }
    }

    pub fn handle_move(&mut self, from: Square, to: Square, promotion: Option<Piece>, ws: &mut Ws) {
        match self {
            ArenaSource::Sandbox(game) => {
                if let Some(mv) = game.find_move(from, to, promotion) {
                    let _ = game.play_move(mv);
                }
            }
            ArenaSource::Active(id) => {
                ws.send(ClientMessage::PlayMove {
                    session_id: *id,
                    mv: PlayMove::Normal(SessionAction::MoveFromTo {
                        from,
                        to,
                        promotion,
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

    pub fn movement(&self, store: &Store, user_id: UserId) -> (bool, Option<Color>) {
        match self {
            Self::Active(id) => {
                if let Some(session) = store.active_sessions.get_entry(id) {
                    let color = if user_id == session.white {
                        Some(Color::White)
                    } else if user_id == session.black {
                        Some(Color::Black)
                    } else {
                        None
                    };
                    if color.is_some() {
                        (session.can_move(user_id), color)
                    } else {
                        (false, None)
                    }
                } else {
                    (false, None)
                }
            }
            Self::Sandbox(_) => (true, None),
        }
    }

    pub fn needs_promotion(&self, from: Square, to: Square, store: &Store) -> bool {
        match self {
            ArenaSource::Sandbox(game) => game.has_promotion_move(from, to),
            ArenaSource::Active(id) => {
                let Some(session) = store.active_sessions.get_entry(id) else {
                    return false;
                };
                session.has_promotion_move(from, to)
            }
        }
    }
}

pub struct ArenaVisuals {
    pub board: BoardVisuals,
    pub top_player: Option<(UserId, Color)>,
    pub bottom_player: Option<(UserId, Color)>,
}
