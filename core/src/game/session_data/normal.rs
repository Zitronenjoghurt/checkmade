use crate::error::{CoreError, CoreResult};
use giga_chess::game::mode::GameMode;
use giga_chess::prelude::clock::ChessClock;
use giga_chess::prelude::config::{SessionConfig, StartingPosition, TimeControl};
use giga_chess::prelude::{ChessMove, Color, GameOutcome, Session, SessionRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bitcode", derive(bitcode::Encode, bitcode::Decode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NormalSessionData {
    pub mode: GameMode,
    pub starting_position: StartingPosition,
    pub time_control: TimeControl,
    pub draw_offer: Option<Color>,
    pub clock: Option<ChessClock>,
    pub moves: Vec<ChessMove>,
    pub outcome: Option<GameOutcome>,
}

impl TryFrom<NormalSessionData> for Session {
    type Error = CoreError;

    fn try_from(value: NormalSessionData) -> CoreResult<Self> {
        let config = SessionConfig {
            mode: value.mode,
            starting_position: value.starting_position,
            time_control: value.time_control,
            pgn: Default::default(),
        };
        let record = SessionRecord {
            config,
            draw_offer: value.draw_offer,
            clock: value.clock,
            moves: value.moves,
            outcome: value.outcome,
        };
        record
            .restore()
            .map_err(|err| CoreError::SessionRestoration(err.to_string()))
    }
}

impl From<Session> for NormalSessionData {
    fn from(value: Session) -> Self {
        let record = value.record();
        Self {
            mode: record.config.mode,
            starting_position: record.config.starting_position,
            time_control: record.config.time_control,
            draw_offer: record.draw_offer,
            clock: record.clock,
            moves: record.moves,
            outcome: record.outcome,
        }
    }
}
