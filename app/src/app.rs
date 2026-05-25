use crate::event::*;
use crate::http::Http;
use crate::i18n::Translatable;
use crate::server_time::ServerTime;
use crate::store::Store;
use crate::ui::icons;
use crate::ui::state::arena::ArenaSource;
use crate::ui::state::UiState;
use crate::ui::tabs::{Tab, TabViewer};
use crate::ui::widgets::connection_status::ConnectionStatus;
use crate::ui::widgets::generic_select::GenericSelect;
use crate::ui::widgets::profile_menu::ProfileMenu;
use crate::ui::widgets::with_badge::WithBadge;
use crate::utils::images::Images;
use crate::ws::Ws;
use checkmade_core::game::play_event::PlayEvent;
use checkmade_core::giga_chess::prelude::event::SessionEvent;
use checkmade_core::giga_chess::prelude::{Color, GameOutcome};
use checkmade_core::lingo::Lingo::*;
use checkmade_core::messages::server::ServerMessage;
use egui::{CentralPanel, Panel, Widget};
use egui_dock::DockState;
use egui_notify::Toasts;
use log::debug;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Checkmade {
    dock: DockState<Tab>,
    ui: UiState,
    #[serde(skip, default)]
    images: Images,
    #[serde(skip, default)]
    server_time: ServerTime,
    #[serde(skip, default)]
    store: Store,
    #[serde(skip, default)]
    toasts: Toasts,
    #[serde(skip, default)]
    http: Http,
    #[serde(skip, default = "default_ws")]
    ws: Ws,
}

impl Default for Checkmade {
    fn default() -> Self {
        Self {
            dock: DockState::new(vec![]),
            ui: UiState::default(),
            images: Images::default(),
            server_time: ServerTime::default(),
            store: Store::default(),
            toasts: Toasts::default(),
            http: Http::default(),
            ws: default_ws(),
        }
    }
}

fn default_ws() -> Ws {
    let mut ws = Ws::default();
    ws.connect(crate::get_ws_url());
    ws
}

impl Checkmade {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        Self::setup_fonts(&cc.egui_ctx);
        cc.storage
            .and_then(|storage| eframe::get_value::<Self>(storage, eframe::APP_KEY))
            .unwrap_or_default()
    }

    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
    }
}

impl eframe::App for Checkmade {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update_arena();
        self.handle_events(ui.ctx());
        self.http.update(&mut self.toasts);
        self.update_ws(ui);
        if self.ws.is_connected() {
            self.server_time.update(ui.ctx(), &mut self.ws);
            self.store.update(ui.ctx(), &mut self.ws);
        }
        self.toasts.show(ui.ctx());
        self.render(ui);
        ui.ctx().request_repaint();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

// Update helpers
impl Checkmade {
    pub fn update_ws(&mut self, ui: &egui::Ui) {
        self.ws.update(ui.ctx());
        let messages = self.ws.drain_incoming().collect::<Vec<_>>();
        for msg in messages {
            self.handle_message(msg);
        }
    }
}

// Message handlers
impl Checkmade {
    pub fn handle_message(&mut self, msg: ServerMessage) {
        debug!("Received {}", msg.name());
        match msg {
            ServerMessage::ActiveSessions(sessions) => {
                self.store
                    .sessions
                    .set_value(sessions.into_iter().map(|s| (s.id, s)).collect());
            }
            ServerMessage::Error(err) => {
                self.toasts.error(err);
            }
            ServerMessage::FriendRequestIncoming(info) => {
                self.store
                    .incoming_friend_requests
                    .insert(info.user_id, info.created);
            }
            ServerMessage::FriendRequestDeclinedByPeer(id) => {
                self.store.outgoing_friend_requests.remove(&id);
            }
            ServerMessage::FriendshipEstablished(info) => {
                self.store.incoming_friend_requests.remove(&info.user_id);
                self.store.outgoing_friend_requests.remove(&info.user_id);
                self.store.friends.insert(info.user_id, info);
                self.toasts.success(FriendAdded.t());
            }
            ServerMessage::FriendshipRemovedByPeer(id) => {
                self.store.friends.remove(&id);
            }
            ServerMessage::FriendRequestSendOk(info) => {
                self.store
                    .outgoing_friend_requests
                    .insert(info.user_id, info.created);
                self.toasts.success(FriendRequestSent.t());
            }
            ServerMessage::FriendRequestDeclineOk(id) => {
                self.store.incoming_friend_requests.remove(&id);
                self.toasts.success(FriendRequestDeclined.t());
            }
            ServerMessage::FriendRequestRemoveOk(id) => {
                self.store.outgoing_friend_requests.remove(&id);
            }
            ServerMessage::FriendRequestRemovedByPeer(id) => {
                self.store.incoming_friend_requests.remove(&id);
            }
            ServerMessage::FriendRemoveOk(id) => {
                self.store.friends.remove(&id);
                self.toasts.success(FriendRemoved.t());
            }
            ServerMessage::Friends(friends) => {
                let map = friends.into_iter().map(|f| (f.user_id, f)).collect();
                self.store.friends.set_value(map);
            }
            ServerMessage::IncomingFriendRequests(requests) => {
                let map = requests
                    .into_iter()
                    .map(|r| (r.user_id, r.created))
                    .collect();
                self.store.incoming_friend_requests.set_value(map);
            }
            ServerMessage::IncomingSessionRequests(requests) => {
                self.store
                    .incoming_session_requests
                    .set_value(requests.into_iter().map(|r| (r.id, r)).collect());
            }
            ServerMessage::OutgoingFriendRequests(requests) => {
                let map = requests
                    .into_iter()
                    .map(|r| (r.user_id, r.created))
                    .collect();
                self.store.outgoing_friend_requests.set_value(map);
            }
            ServerMessage::OutgoingSessionRequests(requests) => {
                self.store
                    .outgoing_session_requests
                    .set_value(requests.into_iter().map(|r| (r.id, r)).collect());
            }
            ServerMessage::Pong {
                client_time,
                server_time,
            } => {
                self.server_time.handle_pong(client_time, server_time);
            }
            ServerMessage::PrivateUserInfo(info) => self.store.me.set_value(info),
            ServerMessage::PublicSessionRequests(requests) => {
                self.store
                    .public_session_requests
                    .set_value(requests.into_iter().map(|r| (r.id, r)).collect());
            }
            ServerMessage::PublicUserInfo(info) => self.store.users.set(info.id, info),
            ServerMessage::Session(session) => {
                if let Some(me) = &self.store.me.value {
                    self.store.sessions.insert(session.id, session);
                }
            }
            ServerMessage::Sessions(sessions) => {
                for session in sessions {
                    self.store.sessions.insert(session.id, session);
                }
            }
            ServerMessage::SessionStart {
                session,
                request_id,
            } => {
                self.store.sessions.insert(session.id, session);
                self.store.incoming_session_requests.remove(&request_id);
                self.store.outgoing_session_requests.remove(&request_id);
            }
            ServerMessage::SessionRequest(request) => {
                if let Some(me) = &self.store.me.value {
                    if request.requester_id == me.public.id {
                        self.store
                            .outgoing_session_requests
                            .insert(request.id, request);
                    } else if let Some(opponent_id) = request.opponent_id
                        && opponent_id == me.public.id
                    {
                        self.store
                            .incoming_session_requests
                            .insert(request.id, request);
                    } else {
                        self.store
                            .public_session_requests
                            .insert(request.id, request);
                    }
                }
            }
            ServerMessage::SessionRequestCreateOk(request) => {
                self.store
                    .outgoing_session_requests
                    .insert(request.id, request);
                self.toasts.success(SessionRequestCreated.t());
            }
            ServerMessage::SessionRequestDeclinedByPeer(id) => {
                self.store.outgoing_session_requests.remove(&id);
            }
            ServerMessage::SessionRequestDeclineOk(id) => {
                self.store.incoming_session_requests.remove(&id);
                self.toasts.success(SessionRequestDeclined.t());
            }
            ServerMessage::SessionRequestIncoming(request) => {
                self.store
                    .incoming_session_requests
                    .insert(request.id, request);
                self.toasts.info(SessionRequestReceived.t());
            }
            ServerMessage::SessionRequestRemoveOk(id) => {
                self.store.outgoing_session_requests.remove(&id);
            }
            ServerMessage::SessionRequestRemovedByPeer(id) => {
                self.store.incoming_session_requests.remove(&id);
            }
            ServerMessage::SessionUpdate {
                session_id,
                color,
                mv,
                at,
            } => {
                let Some(session) = self.store.sessions.get_entry_mut(&session_id) else {
                    self.ws.request_session(session_id);
                    return;
                };
                let Ok(event) = session.play(color, mv, at) else {
                    self.ws.request_session(session_id);
                    return;
                };
                session.updated = at;
                self.handle_play_event(event);
            }
        }
    }
}

// Rendering
impl Checkmade {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        self.ui.update(ui.ctx());

        self.show_top_bar(ui);

        CentralPanel::default().show_inside(ui, |ui| {
            let mut viewer = TabViewer {
                images: &mut self.images,
                state: &mut self.ui,
                server_time: &mut self.server_time,
                store: &mut self.store,
                toasts: &mut self.toasts,
                ws: &mut self.ws,
            };
            egui_dock::DockArea::new(&mut self.dock)
                .style(egui_dock::Style::from_egui(ui.style().as_ref()))
                .show_leaf_collapse_buttons(false)
                .show_leaf_close_all_buttons(false)
                .show_inside(ui, &mut viewer);
        });
    }

    fn show_top_bar(&mut self, ui: &mut egui::Ui) {
        Panel::top("top_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Checkmade");
                ui.separator();
                ConnectionStatus::new(&self.server_time, &self.ws).ui(ui);
                ui.separator();

                if ui
                    .button(icons::GEAR_SIX)
                    .on_hover_text(Settings.t())
                    .clicked()
                {
                    self.open_tab(Tab::Settings);
                }

                let resp = ui
                    .add(
                        WithBadge::new(egui::Button::new(icons::USERS_THREE))
                            .dot(self.store.friend_request_count() > 0),
                    )
                    .on_hover_text(Friends.t());
                if resp.clicked() {
                    self.open_tab(Tab::Friends);
                }

                ui.separator();

                let resp = ui
                    .add(
                        WithBadge::new(egui::Button::new(icons::GAME_CONTROLLER)).dot(
                            self.store.game_request_count() > 0
                                || self.store.active_sessions_to_move_count() > 0,
                        ),
                    )
                    .on_hover_text(Games.t());
                if resp.clicked() {
                    self.open_tab(Tab::Games);
                }

                if ui
                    .button(icons::CHECKERBOARD)
                    .on_hover_text(Arena.t())
                    .clicked()
                {
                    self.open_tab(Tab::Arena);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ProfileMenu::new(&self.store, &mut self.http).ui(ui);
                    self.ui.settings.dirty |=
                        GenericSelect::from_enum(&mut self.ui.settings.locale, "locale_select")
                            .ui(ui)
                            .changed();
                });
            });
        });
    }

    fn open_tab(&mut self, tab: Tab) {
        if let Some(path) = self.dock.find_tab(&tab) {
            let _ = self.dock.set_active_tab(path);
            return;
        }
        self.dock.main_surface_mut().push_to_focused_leaf(tab);
    }
}

// Event handling
impl Checkmade {
    fn handle_events(&mut self, ctx: &egui::Context) {
        flush_all_events(ctx);

        for ErrorEvent(err) in ErrorEvent::recv(ctx) {
            self.toasts.error(err);
        }

        for InfoEvent(info) in InfoEvent::recv(ctx) {
            self.toasts.info(info);
        }

        if DisconnectedEvent::fired(ctx) {
            self.toasts.error(ConnectionLost.t());
        }

        if ReconnectedEvent::fired(ctx) {
            self.toasts.success(ConnectionEstablished.t());
            if let Some(session_id) = self.ui.arena.session_id() {
                self.ws.subscribe_session(session_id);
            }
        }

        for OpenSessionEvent(id) in OpenSessionEvent::recv(ctx) {
            if let Some(old_id) = self.ui.arena.session_id() {
                if old_id != id {
                    self.ui.arena.source = ArenaSource::Active(id);
                }
            } else {
                self.ui.arena.source = ArenaSource::Active(id);
            }
            self.open_tab(Tab::Arena);
        }

        for OpenSandboxEvent(state) in OpenSandboxEvent::recv(ctx) {
            self.ui.arena.source = ArenaSource::Sandbox(state);
            self.open_tab(Tab::Arena);
        }
    }
}

// Play shenanigans
impl Checkmade {
    fn update_arena(&mut self) {
        let Some(current_id) = self.ui.arena.session_id() else {
            if let Some(old_id) = self.ui.arena.subscribed_session {
                self.ws.unsubscribe_session(old_id);
                self.ui.arena.subscribed_session = None;
            }
            return;
        };

        if let Some(old_id) = self.ui.arena.subscribed_session {
            if old_id != current_id {
                self.ws.unsubscribe_session(old_id);
                self.ws.subscribe_session(current_id);
            }
        } else {
            self.ws.subscribe_session(current_id);
        }
        self.ui.arena.subscribed_session = Some(current_id);
    }

    fn handle_play_event(&mut self, event: PlayEvent) {
        match event {
            PlayEvent::Normal(event) => match event {
                SessionEvent::GameOver(outcome) => {
                    if let Some(session_id) = self.ui.arena.session_id() {
                        self.ui.arena.subscribed_session = None;
                        self.ws.unsubscribe_session(session_id);
                        if let Some(session) = self.store.sessions.get_entry(&session_id)
                            && let Some(me) = self.store.me.value.as_ref()
                        {
                            let me_color = if session.white == me.public.id {
                                Some(Color::White)
                            } else if session.black == me.public.id {
                                Some(Color::Black)
                            } else {
                                None
                            };
                            if let Some(me_color) = me_color {
                                let opponent_id = if me_color == Color::White {
                                    session.black
                                } else {
                                    session.white
                                };
                                if let Some(friend_info) =
                                    self.store.friends.get_entry_mut(&opponent_id)
                                {
                                    match outcome {
                                        GameOutcome::Decisive { winner, .. } => {
                                            if winner == me_color {
                                                friend_info.times_won += 1
                                            } else {
                                                friend_info.times_lost += 1
                                            }
                                        }
                                        GameOutcome::Draw(_) => friend_info.times_drawn += 1,
                                    }
                                }
                            }
                        }
                    }
                    self.ui.arena.transform_active_into_sandbox(&self.store);
                }
                SessionEvent::DrawOffered { by } => {}
                _ => {}
            },
        }
    }
}
