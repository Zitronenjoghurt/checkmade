use crate::http::Http;
use crate::i18n::Translatable;
use crate::server_time::ServerTime;
use crate::store::Store;
use crate::ui::icons;
use crate::ui::state::UiState;
use crate::ui::tabs::{Tab, TabViewer};
use crate::ui::widgets::generic_select::GenericSelect;
use crate::ui::widgets::profile_menu::ProfileMenu;
use crate::ui::widgets::with_badge::WithBadge;
use crate::ws::Ws;
use checkmade_core::lingo::Lingo::*;
use checkmade_core::messages::server::ServerMessage;
use egui::{CentralPanel, Panel, Widget};
use egui_dock::DockState;
use egui_notify::Toasts;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Checkmade {
    dock: DockState<Tab>,
    ui: UiState,
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
        self.http.update(&mut self.toasts);
        self.update_ws();
        if self.ws.is_connected() {
            self.server_time.update(&mut self.ws);
            self.store.update(&mut self.ws);
        }
        self.toasts.show(ui.ctx());
        self.render(ui);
        ui.ctx().request_repaint();

        if self.server_time.is_timed_out() && self.ws.is_connected() {
            self.ws.disconnect();
            self.toasts.error("Connection timed out.");
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

// Update helpers
impl Checkmade {
    pub fn update_ws(&mut self) {
        self.ws.update(&mut self.toasts);
        let messages = self.ws.drain_incoming().collect::<Vec<_>>();
        for msg in messages {
            self.handle_message(msg);
        }
    }
}

// Message handlers
impl Checkmade {
    pub fn handle_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Error(err) => {
                self.toasts.error(err);
            }
            ServerMessage::FriendRequestIncoming(info) => {
                self.store
                    .incoming_friend_requests
                    .insert(info.user_id, info.since);
                self.toasts.info(FriendRequestReceived.t());
            }
            ServerMessage::FriendRequestDeclinedByPeer(id) => {
                self.store.outgoing_friend_requests.remove(&id);
            }
            ServerMessage::FriendshipEstablished(info) => {
                self.store.incoming_friend_requests.remove(&info.user_id);
                self.store.outgoing_friend_requests.remove(&info.user_id);
                self.store.friends.insert(info.user_id, info.since);
                self.toasts.success(FriendAdded.t());
            }
            ServerMessage::FriendshipRemovedByPeer(id) => {
                self.store.friends.remove(&id);
            }
            ServerMessage::FriendRequestSendOk(info) => {
                self.store
                    .outgoing_friend_requests
                    .insert(info.user_id, info.since);
                self.toasts.success(FriendRequestSent.t());
            }
            ServerMessage::FriendRequestDeclineOk(id) => {
                self.store.incoming_friend_requests.remove(&id);
                self.toasts.success(FriendRequestDeclined.t());
            }
            ServerMessage::FriendRemoveOk(id) => {
                self.store.friends.remove(&id);
                self.toasts.success(FriendRemoved.t());
            }
            ServerMessage::Friends(friends) => {
                let map = friends.into_iter().map(|f| (f.user_id, f.since)).collect();
                self.store.friends.set_value(map);
            }
            ServerMessage::IncomingFriendRequests(requests) => {
                let map = requests.into_iter().map(|r| (r.user_id, r.since)).collect();
                self.store.incoming_friend_requests.set_value(map);
            }
            ServerMessage::OutgoingFriendRequests(requests) => {
                let map = requests.into_iter().map(|r| (r.user_id, r.since)).collect();
                self.store.outgoing_friend_requests.set_value(map);
            }
            ServerMessage::Pong {
                client_time,
                server_time,
            } => {
                self.server_time.handle_pong(client_time, server_time);
            }
            ServerMessage::PrivateUserInfo(info) => self.store.me.set_value(info),
            ServerMessage::PublicUserInfo(info) => self.store.users.set(info.id, info),
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
