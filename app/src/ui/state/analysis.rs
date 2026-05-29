use crate::event::{AppEvent, SfCommandEvent, SfResponseEvent};
use checkmade_core::giga_chess::prelude::{ChessMove, Square};
use checkmade_core::giga_chess::stockfish::command::{SfCommand, SfGo, SfPosition};
use checkmade_core::giga_chess::stockfish::event::{SfEvent, SfScore};
use std::collections::VecDeque;
use std::str::FromStr;

const BACKFILL_DEPTH: u64 = 14;

pub struct AnalysisState {
    enabled: bool,
    pub depth: u64,

    pub eval: Option<EvalInfo>,
    pub eval_history: Vec<Option<EvalPoint>>,

    game_moves: Vec<ChessMove>,
    viewed_ply: usize,
    last_live_key: Option<usize>,

    mode: EvalMode,
    backfill_queue: VecDeque<usize>,
    backfill_score: Option<SfScore>,
    skip_next_best_move: bool,
}

impl Default for AnalysisState {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: 20,
            eval: None,
            eval_history: Vec::new(),
            game_moves: Vec::new(),
            viewed_ply: 0,
            last_live_key: None,
            mode: EvalMode::Idle,
            backfill_queue: VecDeque::new(),
            backfill_score: None,
            skip_next_best_move: false,
        }
    }
}

impl AnalysisState {
    pub fn sync_game(&mut self, all_moves: &[ChessMove], viewed_ply: usize, ctx: &egui::Context) {
        if !self.enabled {
            return;
        }

        let game_changed = self.game_moves.as_slice() != all_moves;
        let view_changed = self.viewed_ply != viewed_ply;

        if game_changed {
            self.game_moves = all_moves.to_vec();

            let needed = all_moves.len() + 1;
            self.eval_history.resize_with(needed, || None);
            self.eval_history.truncate(needed);
        }

        if game_changed || view_changed {
            self.viewed_ply = viewed_ply;
            self.start_live_eval(ctx);
        }

        if game_changed {
            self.rebuild_backfill_queue();
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if !self.enabled {
            return;
        }
        for SfResponseEvent(event) in SfResponseEvent::recv(ctx) {
            self.handle_event(event, ctx);
        }
    }

    pub fn toggle(&mut self, ctx: &egui::Context) {
        self.enabled = !self.enabled;
        if !self.enabled {
            SfCommandEvent(SfCommand::Stop).send(ctx);
            self.eval = None;
            self.reset_internals();
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn invalidate(&mut self) {
        self.last_live_key = None;
    }

    fn handle_event(&mut self, event: SfEvent, ctx: &egui::Context) {
        if self.skip_next_best_move {
            if matches!(event, SfEvent::BestMove { .. }) {
                self.skip_next_best_move = false;
            }
            return;
        }

        match &self.mode {
            EvalMode::Live => match event {
                SfEvent::Info(info) => {
                    if let Some(score) = info.score {
                        let best_move = info.pv.first().and_then(|uci| parse_uci_squares(uci));
                        self.eval = Some(EvalInfo {
                            score,
                            depth: info.depth.unwrap_or(0),
                            best_move,
                            pv_uci: info.pv,
                            wdl: info.wdl.map(|w| (w.win, w.draw, w.loss)),
                        });
                    }
                }
                SfEvent::BestMove { mv, .. } => {
                    let snapshot = if let Some(eval) = &mut self.eval {
                        eval.best_move = parse_uci_squares(&mv);
                        Some((eval.score.clone(), eval.depth))
                    } else {
                        None
                    };

                    if let Some((score, depth)) = snapshot {
                        self.store_history_point(self.viewed_ply, &score, depth);
                    }

                    self.start_next_backfill(ctx);
                }
                _ => {}
            },
            EvalMode::Backfill(ply) => {
                let ply = *ply;
                match event {
                    SfEvent::Info(info) => {
                        if let Some(score) = info.score {
                            self.backfill_score = Some(score);
                        }
                    }
                    SfEvent::BestMove { .. } => {
                        if let Some(score) = self.backfill_score.take() {
                            let depth = BACKFILL_DEPTH as u32;
                            self.store_history_point(ply, &score, depth);
                        }
                        self.start_next_backfill(ctx);
                    }
                    _ => {}
                }
            }
            EvalMode::Idle => {}
        }
    }

    fn start_live_eval(&mut self, ctx: &egui::Context) {
        if !matches!(self.mode, EvalMode::Idle) {
            self.skip_next_best_move = true;
        }

        self.last_live_key = Some(self.viewed_ply);
        self.eval = None;
        self.mode = EvalMode::Live;

        self.send_position_cmd(self.viewed_ply, self.depth, ctx);
    }

    fn rebuild_backfill_queue(&mut self) {
        self.backfill_queue.clear();
        for i in 0..self.eval_history.len() {
            if i != self.viewed_ply && self.eval_history[i].is_none() {
                self.backfill_queue.push_back(i);
            }
        }
    }

    fn start_next_backfill(&mut self, ctx: &egui::Context) {
        while let Some(&ply) = self.backfill_queue.front() {
            if ply < self.eval_history.len() && self.eval_history[ply].is_none() {
                break;
            }
            self.backfill_queue.pop_front();
        }

        if let Some(ply) = self.backfill_queue.pop_front() {
            self.backfill_score = None;
            self.mode = EvalMode::Backfill(ply);
            self.send_position_cmd(ply, BACKFILL_DEPTH, ctx);
        } else {
            self.mode = EvalMode::Idle;
        }
    }

    fn send_position_cmd(&self, ply: usize, depth: u64, ctx: &egui::Context) {
        SfCommandEvent(SfCommand::Stop).send(ctx);
        SfCommandEvent(SfCommand::Position(SfPosition {
            fen: None,
            moves: self.game_moves[..ply]
                .iter()
                .map(|m| m.to_string())
                .collect(),
        }))
        .send(ctx);
        SfCommandEvent(SfCommand::Go(SfGo {
            depth: if depth > 0 { Some(depth) } else { None },
            infinite: depth == 0,
            ..Default::default()
        }))
        .send(ctx);
    }

    fn store_history_point(&mut self, ply: usize, score: &SfScore, depth: u32) {
        if ply >= self.eval_history.len() {
            return;
        }
        let mut cp = score_to_cp(score);
        if ply % 2 == 1 {
            cp = -cp;
        }
        self.eval_history[ply] = Some(EvalPoint {
            cp,
            score: score.clone(),
            depth,
        });
    }

    fn reset_internals(&mut self) {
        self.eval_history.clear();
        self.game_moves.clear();
        self.last_live_key = None;
        self.mode = EvalMode::Idle;
        self.backfill_queue.clear();
        self.backfill_score = None;
        self.skip_next_best_move = false;
    }

    pub fn viewed_ply(&self) -> usize {
        self.viewed_ply
    }

    pub fn is_thinking(&self) -> bool {
        matches!(self.mode, EvalMode::Live | EvalMode::Backfill(_))
    }
}

#[derive(Debug, Default)]
enum EvalMode {
    #[default]
    Idle,
    Live,
    Backfill(usize),
}

#[derive(Debug, Clone)]
pub struct EvalPoint {
    pub cp: f64,
    pub score: SfScore,
    pub depth: u32,
}

#[derive(Debug)]
pub struct EvalInfo {
    pub score: SfScore,
    pub depth: u32,
    pub best_move: Option<(Square, Square)>,
    pub pv_uci: Vec<String>,
    pub wdl: Option<(u32, u32, u32)>,
}

fn parse_uci_squares(uci: &str) -> Option<(Square, Square)> {
    if uci.len() < 4 {
        return None;
    }
    let from = Square::from_str(&uci[0..2]).ok()?;
    let to = Square::from_str(&uci[2..4]).ok()?;
    Some((from, to))
}

pub fn score_to_cp(score: &SfScore) -> f64 {
    match score {
        SfScore::Cp { value, .. } => *value as f64,
        SfScore::Mate { value, .. } => {
            if *value > 0 {
                10_000.0
            } else {
                -10_000.0
            }
        }
    }
}

pub fn format_score(score: &SfScore) -> String {
    match score {
        SfScore::Cp { value, .. } => format!("{:+.2}", *value as f64 / 100.0),
        SfScore::Mate { value, .. } => format!("M{}", value),
    }
}
