use crate::ui::icons;
use strum::EnumIter;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub ui_scale: f32,
    pub current_tab: SettingsTab,
    #[serde(skip, default = "default_true")]
    pub dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_scale: Self::DEFAULT_UI_SCALE,
            current_tab: SettingsTab::default(),
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
    pub fn title(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingsTab::General => icons::GEAR_SIX,
        }
    }
}
