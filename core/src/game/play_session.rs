use crate::error::{CoreError, CoreResult, DomainError};
use crate::game::misc_action::MiscAction;
use crate::game::play_event::PlayEvent;
use crate::game::play_move::PlayMove;
use crate::game::session_data::{SessionConfigData, SessionData};
use crate::game::visuals::BoardVisuals;
use crate::types::session_id::SessionId;
use crate::types::session_status::SessionStatus;
use crate::types::user_id::UserId;
use giga_chess::prelude::action::SessionAction;
use giga_chess::prelude::{Color, GameOutcome, Piece, Session};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlaySession {
    pub id: SessionId,
    pub public: bool,
    pub white: UserId,
    pub black: UserId,
    pub created: u64,
    pub updated: u64,
    pub kind: PlaySessionKind,
}

impl PlaySession {
    pub fn play(&mut self, color: Color, mv: PlayMove, unix_ms: u64) -> CoreResult<PlayEvent> {
        let event = match &mut self.kind {
            PlaySessionKind::Normal(session) => {
                let PlayMove::Normal(action) = mv else {
                    return Err(DomainError::InvalidMove.into());
                };
                PlayEvent::Normal(
                    session
                        .act(color, action, unix_ms)
                        .map_err(DomainError::Session)?,
                )
            }
        };
        Ok(event)
    }

    pub fn can_move(&self, user_id: UserId) -> bool {
        if !self.is_ongoing() || (user_id != self.white && user_id != self.black) {
            return false;
        };

        match &self.kind {
            PlaySessionKind::Normal(session) => {
                let color = session.game().position().side_to_move;
                (color == Color::White && user_id == self.white)
                    || (color == Color::Black && user_id == self.black)
            }
        }
    }

    pub fn move_count(&self) -> usize {
        match &self.kind {
            PlaySessionKind::Normal(session) => {
                if session.game().position().side_to_move == Color::White {
                    ((session.game().position().full_moves as usize) * 2) - 1
                } else {
                    (session.game().position().full_moves as usize) * 2
                }
            }
        }
    }

    pub fn status(&self) -> SessionStatus {
        match &self.kind {
            PlaySessionKind::Normal(session) => match session.game().outcome() {
                None => SessionStatus::Ongoing,
                Some(outcome) => match outcome {
                    GameOutcome::Draw(_) => SessionStatus::Draw,
                    GameOutcome::Decisive { winner, .. } => {
                        if winner == Color::White {
                            SessionStatus::WhiteWins
                        } else {
                            SessionStatus::BlackWins
                        }
                    }
                },
            },
        }
    }

    pub fn is_ongoing(&self) -> bool {
        matches!(self.status(), SessionStatus::Ongoing)
    }

    pub fn visuals(&self, user_id: UserId, color: Color) -> BoardVisuals {
        match &self.kind {
            PlaySessionKind::Normal(session) => {
                let perspective = if user_id == self.white {
                    Color::White
                } else if user_id == self.black {
                    Color::Black
                } else {
                    color
                };
                BoardVisuals::from_game(perspective, session.game())
            }
        }
    }

    pub fn captured_pieces(&self, color: Color) -> &[Piece] {
        match &self.kind {
            PlaySessionKind::Normal(session) => session.game().captured_pieces(color),
        }
    }

    pub fn has_promotion_move(
        &self,
        from: giga_chess::prelude::Square,
        to: giga_chess::prelude::Square,
    ) -> bool {
        match &self.kind {
            PlaySessionKind::Normal(session) => session.game().has_promotion_move(from, to),
        }
    }

    pub fn game(&self) -> &giga_chess::prelude::Game {
        match &self.kind {
            PlaySessionKind::Normal(session) => session.game(),
        }
    }

    pub fn draw_offer(&self) -> Option<Color> {
        match &self.kind {
            PlaySessionKind::Normal(session) => session.draw_offer(),
        }
    }

    pub fn wrap_misc_action(&self, action: MiscAction) -> PlayMove {
        match &self.kind {
            PlaySessionKind::Normal(_) => match action {
                MiscAction::Resign => PlayMove::Normal(SessionAction::Resign),
                MiscAction::OfferDraw => PlayMove::Normal(SessionAction::OfferDraw),
                MiscAction::AcceptDraw => PlayMove::Normal(SessionAction::AcceptDraw),
                MiscAction::DeclineDraw => PlayMove::Normal(SessionAction::DeclineDraw),
                MiscAction::ClaimDraw => PlayMove::Normal(SessionAction::ClaimDraw),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlaySessionKind {
    Normal(Session),
}

impl TryFrom<SessionData> for PlaySessionKind {
    type Error = CoreError;

    fn try_from(value: SessionData) -> CoreResult<Self> {
        match value {
            SessionData::Normal(normal) => Ok(PlaySessionKind::Normal(normal.try_into()?)),
        }
    }
}

impl From<PlaySessionKind> for SessionData {
    fn from(value: PlaySessionKind) -> Self {
        match value {
            PlaySessionKind::Normal(session) => SessionData::Normal(session.into()),
        }
    }
}

impl TryFrom<SessionConfigData> for PlaySessionKind {
    type Error = CoreError;

    fn try_from(config: SessionConfigData) -> CoreResult<Self> {
        match config {
            SessionConfigData::Normal(config) => Ok(PlaySessionKind::Normal(
                Session::from_config(&config)
                    .map_err(|err| CoreError::SessionCreation(err.to_string()))?,
            )),
        }
    }
}
