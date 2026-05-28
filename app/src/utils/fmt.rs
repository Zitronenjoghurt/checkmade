use crate::i18n::Translatable;
use crate::tl;
use checkmade_core::giga_chess::prelude::{Color, DecisiveReason, DrawReason, GameOutcome};
use checkmade_core::lingo::Lingo::*;

pub fn fmt_duration(duration: web_time::Duration) -> String {
    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    match (days, hours, minutes, seconds, millis) {
        (0, 0, 0, 0, ms) => format!("{ms}ms"),
        (0, 0, 0, s, 0) => format!("{s}s"),
        (0, 0, 0, s, ms) => format!("{s}s {ms}ms"),
        (0, 0, m, s, _) => format!("{m}m {s}s"),
        (0, h, m, s, _) => format!("{h}h {m}m {s}s"),
        (d, h, m, _, _) => format!("{d}d {h}h {m}m"),
    }
}

pub fn fmt_color(color: Color) -> String {
    match color {
        Color::White => White.t().to_string(),
        Color::Black => Black.t().to_string(),
    }
}

pub fn fmt_outcome(outcome: &GameOutcome) -> String {
    match outcome {
        GameOutcome::Decisive { winner, reason } => {
            let color = fmt_color(*winner);
            let reason = match reason {
                DecisiveReason::Checkmate => Checkmate.t(),
                DecisiveReason::Resignation => Resignation.t(),
                DecisiveReason::Timeout => Timeout.t(),
            };
            format!("{} {reason}", tl!(XWinsBy, x = color))
        }
        GameOutcome::Draw(reason) => {
            let reason = match reason {
                DrawReason::Stalemate => Stalemate.t(),
                DrawReason::Agreement => Agreement.t(),
                DrawReason::FiftyMoveRule => FiftyMoveRule.t(),
                DrawReason::SeventyFiveMoveRule => SeventyFiveMoveRule.t(),
                DrawReason::ThreefoldRepetition => ThreefoldRepetition.t(),
                DrawReason::FivefoldRepetition => FivefoldRepetition.t(),
                DrawReason::InsufficientMaterial => InsufficientMaterial.t(),
                DrawReason::TimeoutVsInsufficient => TimeoutVsInsufficient.t(),
            };
            tl!(DrawByX, x = reason).to_string()
        }
    }
}

pub fn fmt_clock(time_ms: u64, increment_ms: u64) -> String {
    let total_seconds = time_ms / 1_000;

    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    let tenths = (time_ms % 1_000) / 100;

    if days > 0 {
        let inc_hours = increment_ms / 36_000_000;
        format!("{}d {:02}h +{}h", days, hours, inc_hours)
    } else if hours > 0 {
        let inc_secs = increment_ms / 1_000;
        format!("{}:{:02}:{:02} +{}", hours, minutes, seconds, inc_secs)
    } else if minutes > 0 || seconds >= 10 {
        let inc_secs = increment_ms / 1_000;
        format!("{}:{:02} +{}", minutes, seconds, inc_secs)
    } else {
        let inc_secs = increment_ms / 1_000;
        format!("{}.{} +{}", seconds, tenths, inc_secs)
    }
}
