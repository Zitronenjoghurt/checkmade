use crate::store::Store;
use crate::ui::state::move_history::MoveHistoryState;
use crate::ui::state::sandbox::SandboxState;
use crate::ws::Ws;
use checkmade_core::game::misc_action::MiscAction;
use checkmade_core::game::play_move::PlayMove;
use checkmade_core::game::visuals::BoardVisuals;
use checkmade_core::giga_chess::prelude::action::SessionAction;
use checkmade_core::giga_chess::prelude::state::GameState;
use checkmade_core::giga_chess::prelude::{Color, Game, GameOutcome, Piece, Square};
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
    #[serde(skip, default)]
    pub pending_action: Option<MiscAction>,
    #[serde(skip, default)]
    pub move_history: MoveHistoryState,
}

impl Default for ArenaState {
    fn default() -> Self {
        Self {
            subscribed_session: None,
            perspective: Color::White,
            source: ArenaSource::default(),
            pending_promotion: None,
            pending_action: None,
            move_history: Default::default(),
        }
    }
}

impl ArenaState {
    pub fn visuals(&self, user_id: UserId, store: &Store) -> Option<ArenaVisuals> {
        self.source.visuals(
            user_id,
            self.perspective,
            store,
            self.move_history.current_index(),
        )
    }

    pub fn current_game(&self, store: &Store) -> Option<Game> {
        match &self.source {
            ArenaSource::Sandbox(state) => match self.move_history.current_index() {
                Some(idx) => Some(state.game_at_index(idx)),
                None => Some(state.game.clone()),
            },
            ArenaSource::Active(id) => store.sessions.get_entry(id).map(|s| s.game().clone()),
        }
    }

    pub fn handle_move(&mut self, from: Square, to: Square, promotion: Option<Piece>, ws: &mut Ws) {
        let idx = self.move_history.current_index();
        self.source.handle_move(from, to, promotion, idx, ws);
        self.move_history.snap_to_present();
    }

    pub fn has_session(&self, id: SessionId) -> bool {
        self.source.has_session(id)
    }

    pub fn session_id(&self) -> Option<SessionId> {
        self.source.session_id()
    }

    pub fn movement(&self, store: &Store, user_id: UserId) -> (bool, Option<Color>) {
        if !self.move_history.is_at_present() {
            return match &self.source {
                ArenaSource::Active(_) => (false, None),
                ArenaSource::Sandbox(state) => {
                    let game = match self.move_history.current_index() {
                        Some(idx) => state.game_at_index(idx),
                        None => state.game.clone(),
                    };
                    (true, Some(game.position().side_to_move))
                }
            };
        }
        self.source.movement(store, user_id)
    }

    pub fn captured_pieces<'a>(&'a self, store: &'a Store, color: Color) -> &'a [Piece] {
        self.source.captured_pieces(store, color)
    }

    pub fn transform_active_into_sandbox(&mut self, store: &Store) {
        let Some(me_id) = store.me.value.as_ref().map(|i| i.public.id) else {
            return;
        };

        let ArenaSource::Active(id) = self.source else {
            return;
        };

        let Some(session) = store.sessions.get_entry(&id) else {
            return;
        };

        let perspective = if session.white == me_id {
            Color::White
        } else if session.black == me_id {
            Color::Black
        } else {
            self.perspective
        };

        let sandbox = SandboxState {
            game: session.game().clone(),
            black_id: Some(session.black),
            white_id: Some(session.white),
            perspective,
            san_history: session.san_history().to_vec(),
            previous_lines: vec![],
        };
        self.source = ArenaSource::Sandbox(sandbox);
    }

    pub fn outcome(&self, store: &Store) -> Option<GameOutcome> {
        self.source.outcome(store)
    }

    pub fn color_to_move(&self, store: &Store) -> Option<Color> {
        self.source.color_to_move(store)
    }

    pub fn actions(&self, store: &Store, user_id: UserId) -> ArenaActions {
        self.source.actions(store, user_id)
    }

    pub fn handle_action(&mut self, action: MiscAction, store: &Store, ws: &mut Ws) {
        self.source.handle_action(action, store, ws);
    }

    pub fn time(&self, store: &Store, color: Color, now_ms: u64) -> Option<(u64, u64)> {
        self.source.time(store, color, now_ms)
    }

    pub fn legal_targets(&self, sq: Square, store: &Store) -> Vec<Square> {
        self.source
            .legal_targets(sq, store, self.move_history.current_index())
    }

    pub fn is_sandbox(&self) -> bool {
        matches!(self.source, ArenaSource::Sandbox(_))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ArenaSource {
    Active(SessionId),
    Sandbox(SandboxState),
}

impl Default for ArenaSource {
    fn default() -> Self {
        Self::Sandbox(SandboxState::default())
    }
}

impl ArenaSource {
    pub fn visuals(
        &self,
        user_id: UserId,
        perspective: Color,
        store: &Store,
        move_index: Option<usize>,
    ) -> Option<ArenaVisuals> {
        match self {
            Self::Active(id) => {
                let session = store.sessions.get_entry(id)?;
                let board = session.visuals(user_id, perspective, move_index);
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
            Self::Sandbox(state) => {
                let board = if let Some(move_index) = move_index {
                    BoardVisuals::from_game_at(state.perspective, &state.game, move_index)
                } else {
                    BoardVisuals::from_game(state.perspective, &state.game)
                };
                Some(ArenaVisuals {
                    board,
                    top_player: state.top_player(),
                    bottom_player: state.bottom_player(),
                })
            }
        }
    }

    pub fn handle_move(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<Piece>,
        move_index: Option<usize>,
        ws: &mut Ws,
    ) {
        match self {
            ArenaSource::Sandbox(state) => {
                if let Some(idx) = move_index {
                    state.fork_at(idx);
                }
                if let Some(mv) = state.game.find_move(from, to, promotion) {
                    if let Ok(san) = state.game.play_move_get_san(mv) {
                        state.san_history.push(san);
                    }
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
                if let Some(session) = store.sessions.get_entry(id) {
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
            Self::Sandbox(state) => {
                let side = state.game.position().side_to_move;
                (true, Some(side))
            }
        }
    }

    pub fn captured_pieces<'a>(&'a self, store: &'a Store, color: Color) -> &'a [Piece] {
        match self {
            ArenaSource::Sandbox(state) => state.game.captured_pieces(color),
            ArenaSource::Active(id) => {
                let Some(session) = store.sessions.get_entry(id) else {
                    return &[];
                };
                session.captured_pieces(color)
            }
        }
    }

    pub fn outcome(&self, store: &Store) -> Option<GameOutcome> {
        match self {
            ArenaSource::Active(id) => {
                let session = store.sessions.get_entry(id)?;
                session.game().outcome()
            }
            ArenaSource::Sandbox(state) => state.game.outcome(),
        }
    }

    pub fn color_to_move(&self, store: &Store) -> Option<Color> {
        match self {
            ArenaSource::Active(id) => {
                let session = store.sessions.get_entry(id)?;
                Some(session.game().position().side_to_move)
            }
            ArenaSource::Sandbox(state) => Some(state.game.position().side_to_move),
        }
    }

    pub fn actions(&self, store: &Store, user_id: UserId) -> ArenaActions {
        match self {
            Self::Sandbox(state) => {
                let claimable = matches!(
                    state.game.state(),
                    GameState::DrawFiftyMoveClaimable | GameState::DrawRepetitionClaimable
                );
                ArenaActions {
                    can_resign: false,
                    can_offer_draw: false,
                    can_accept_or_decline_draw: false,
                    can_claim_draw: claimable,
                }
            }
            Self::Active(id) => {
                let Some(session) = store.sessions.get_entry(id) else {
                    return ArenaActions::default();
                };

                if !session.is_ongoing() {
                    return ArenaActions::default();
                }

                let my_color = if user_id == session.white {
                    Color::White
                } else if user_id == session.black {
                    Color::Black
                } else {
                    return ArenaActions::default();
                };
                ArenaActions {
                    can_resign: true,
                    can_offer_draw: session.draw_offer() != Some(my_color),
                    can_accept_or_decline_draw: session.draw_offer() == Some(my_color.opposite()),
                    can_claim_draw: session.can_move(user_id) && {
                        let state = session.game().state();
                        matches!(
                            state,
                            GameState::DrawFiftyMoveClaimable | GameState::DrawRepetitionClaimable
                        )
                    },
                }
            }
        }
    }

    pub fn handle_action(&mut self, action: MiscAction, store: &Store, ws: &mut Ws) {
        match self {
            ArenaSource::Sandbox(state) => match action {
                MiscAction::ClaimDraw => {
                    let _ = state.game.claim_draw();
                }
                MiscAction::Resign => {
                    state.game.resign(state.game.position().side_to_move);
                }
                _ => {}
            },
            ArenaSource::Active(id) => {
                let Some(session) = store.sessions.get_entry(id) else {
                    return;
                };
                let mv = session.wrap_misc_action(action);
                ws.send(ClientMessage::PlayMove {
                    session_id: *id,
                    mv,
                });
            }
        }
    }

    pub fn time(&self, store: &Store, color: Color, now_ms: u64) -> Option<(u64, u64)> {
        match self {
            Self::Active(id) => {
                let Some(session) = store.sessions.get_entry(id) else {
                    return None;
                };
                Some((
                    session.time_left(color, now_ms)?,
                    session.increment_ms(color)?,
                ))
            }
            Self::Sandbox(_) => None,
        }
    }

    pub fn legal_targets(
        &self,
        sq: Square,
        store: &Store,
        move_index: Option<usize>,
    ) -> Vec<Square> {
        match self {
            Self::Sandbox(state) => match move_index {
                Some(idx) => state.game_at_index(idx).legal_targets(sq),
                None => state.game.legal_targets(sq),
            },
            Self::Active(id) => {
                let Some(session) = store.sessions.get_entry(id) else {
                    return vec![];
                };
                session.game().legal_targets(sq)
            }
        }
    }

    pub fn needs_promotion(
        &self,
        from: Square,
        to: Square,
        store: &Store,
        move_index: Option<usize>,
    ) -> bool {
        match self {
            ArenaSource::Sandbox(state) => match move_index {
                Some(idx) => state.game_at_index(idx).has_promotion_move(from, to),
                None => state.game.has_promotion_move(from, to),
            },
            ArenaSource::Active(id) => {
                let Some(session) = store.sessions.get_entry(id) else {
                    return false;
                };
                session.has_promotion_move(from, to)
            }
        }
    }

    pub fn san_history<'a>(&'a self, store: &'a Store) -> &'a [String] {
        match self {
            Self::Active(id) => {
                let Some(session) = store.sessions.get_entry(id) else {
                    return &[];
                };
                session.san_history()
            }
            Self::Sandbox(state) => &state.san_history,
        }
    }
}

pub struct ArenaVisuals {
    pub board: BoardVisuals,
    pub top_player: Option<(UserId, Color)>,
    pub bottom_player: Option<(UserId, Color)>,
}

#[derive(Default)]
pub struct ArenaActions {
    pub can_resign: bool,
    pub can_offer_draw: bool,
    pub can_accept_or_decline_draw: bool,
    pub can_claim_draw: bool,
}
