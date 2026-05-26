use crate::i18n::Translatable;
use checkmade_core::lingo::Lingo::*;
use egui::{Ui, WidgetText};
use strum::EnumIter;

mod arena;
mod changelog;
mod friends;
mod games;
mod settings;

pub struct TabViewer<'a> {
    pub images: &'a mut crate::utils::images::Images,
    pub md: &'a mut crate::utils::markdown::Markdown,
    pub state: &'a mut crate::ui::state::UiState,
    pub server_time: &'a mut crate::server_time::ServerTime,
    pub store: &'a mut crate::store::Store,
    pub toasts: &'a mut egui_notify::Toasts,
    pub ws: &'a mut crate::ws::Ws,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Arena => arena::show(self, ui),
            Tab::Changelog => changelog::show(self, ui),
            Tab::Friends => friends::show(self, ui),
            Tab::Games => games::show(self, ui),
            Tab::Settings => settings::show(self, ui),
        }
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        true
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumIter)]
pub enum Tab {
    Arena,
    Changelog,
    Friends,
    Games,
    Settings,
}

impl Tab {
    pub fn title(&self) -> String {
        match self {
            Tab::Arena => Arena.t().to_string(),
            Tab::Changelog => Changelog.t().to_string(),
            Tab::Friends => Friends.t().to_string(),
            Tab::Games => Games.t().to_string(),
            Tab::Settings => Settings.t().to_string(),
        }
    }
}
