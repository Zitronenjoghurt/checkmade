use crate::ui::state::sandbox::SandboxState;
use crate::ui::widgets::board::{board_action, BoardAction, BoardWidget};
use checkmade_core::giga_chess::game::Game;
use egui::{Response, Ui, Widget};

pub struct SandboxWidget<'a> {
    images: &'a mut crate::utils::images::Images,
    state: &'a mut SandboxState,
}

impl<'a> SandboxWidget<'a> {
    pub fn new(images: &'a mut crate::utils::images::Images, state: &'a mut SandboxState) -> Self {
        Self { images, state }
    }
}

impl Widget for SandboxWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let size = ui.available_width().min(ui.available_height());

            ui.horizontal(|ui| {
                if ui.button("Reset").clicked() {
                    self.state.game = Game::default();
                }
                if ui.button("Flip").clicked() {
                    self.state.perspective = self.state.perspective.opposite();
                }
            });

            BoardWidget::new(self.images, &self.state.visuals(), "sandbox_board")
                .size(size * 0.95)
                .ui(ui);
            if let Some(BoardAction::Move { from, to }) = board_action(ui, "sandbox_board")
                && let Some(mv) = self.state.game.find_move(from, to, None)
            {
                self.state.game.play_move(mv).unwrap();
            }
        })
        .response
    }
}
