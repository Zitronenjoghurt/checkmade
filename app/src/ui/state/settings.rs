use crate::i18n::{Locale, Translatable};
use crate::ui::icons;
use checkmade_core::lingo::Lingo::*;
use strum::EnumIter;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub ui_scale: f32,
    pub display_legal_targets: bool,
    pub current_tab: SettingsTab,
    pub locale: Locale,
    #[serde(skip, default = "default_true")]
    pub dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_scale: Self::DEFAULT_UI_SCALE,
            display_legal_targets: false,
            current_tab: SettingsTab::default(),
            locale: Locale::default(),
            dirty: true,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Settings {
    pub const DEFAULT_UI_SCALE: f32 = 1.5;

    pub fn apply(&mut self, ctx: &egui::Context) {
        if !self.dirty {
            return;
        }

        ctx.set_pixels_per_point(self.ui_scale);
        self.locale.apply();

        self.dirty = false;
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, EnumIter,
)]
pub enum SettingsTab {
    #[default]
    General,
}

impl SettingsTab {
    pub fn title(&self) -> String {
        match self {
            SettingsTab::General => General.t().to_string(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingsTab::General => icons::GEAR_SIX,
        }
    }
}
