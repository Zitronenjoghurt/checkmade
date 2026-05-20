use egui::{Ui, WidgetText};
use strum::EnumIter;

mod settings;

pub struct TabViewer<'a> {
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
            Tab::Settings => settings::show(self, ui),
        }
    }

    fn is_closeable(&self, tab: &Self::Tab) -> bool {
        true
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumIter)]
pub enum Tab {
    Settings,
}

impl Tab {
    pub fn title(&self) -> &'static str {
        match self {
            Tab::Settings => "Settings",
        }
    }
}
