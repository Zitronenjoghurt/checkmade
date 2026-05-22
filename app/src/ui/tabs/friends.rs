use crate::ui::tabs::TabViewer;
use crate::ui::widgets::friends::add::FriendAdd;
use crate::ui::widgets::friends::bar::FriendsBar;
use crate::ui::widgets::friends::incoming::FriendIncoming;
use crate::ui::widgets::friends::list::Friendlist;
use crate::ui::widgets::friends::outgoing::FriendOutgoing;
use crate::ui::widgets::friends::FriendsTab;
use egui::Widget;

pub fn show(v: &mut TabViewer, ui: &mut egui::Ui) {
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                FriendsBar::new(&mut v.state.friends_tab, v.store.friend_request_count()).ui(ui);

                ui.separator();

                match v.state.friends_tab {
                    FriendsTab::List => {
                        Friendlist::new(v.server_time, v.store, v.ws).ui(ui);
                    }
                    FriendsTab::Incoming => {
                        FriendIncoming::new(v.server_time, v.store, v.ws).ui(ui);
                    }
                    FriendsTab::Outgoing => {
                        FriendOutgoing::new(v.server_time, v.store).ui(ui);
                    }
                    FriendsTab::AddFriend => {
                        FriendAdd::new(v.store, v.toasts, v.ws).ui(ui);
                    }
                }
            });
        });
}
