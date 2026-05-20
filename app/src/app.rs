#[derive(serde::Serialize, serde::Deserialize)]
pub struct Checkmade {}

impl Default for Checkmade {
    fn default() -> Self {
        Self {}
    }
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {}

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
