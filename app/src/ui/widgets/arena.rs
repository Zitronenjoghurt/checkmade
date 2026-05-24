use crate::store::Store;
use crate::ui::state::arena::ArenaState;
use crate::ui::widgets::board::{board_action, BoardAction, BoardWidget};
use crate::utils::images::Images;
use crate::ws::Ws;

pub struct ArenaWidget<'a> {
    images: &'a mut Images,
    state: &'a mut ArenaState,
    store: &'a mut Store,
    ws: &'a mut Ws,
}

impl<'a> ArenaWidget<'a> {
    pub fn new(
        images: &'a mut Images,
        state: &'a mut ArenaState,
        store: &'a mut Store,
        ws: &'a mut Ws,
    ) -> Self {
        Self {
            images,
            state,
            store,
            ws,
        }
    }
}

impl egui::Widget for ArenaWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Some(user_id) = self.store.me.value.as_ref().map(|v| v.public.id) else {
            return ui.spinner();
        };

        let Some(visuals) = self.state.visuals(user_id, self.store) else {
            return ui.spinner();
        };

        ui.vertical(|ui| {
            let size = ui.available_width().min(ui.available_height());
            BoardWidget::new(self.images, &visuals, "arena_board")
                .size(size * 0.95)
                .ui(ui);
            if let Some(BoardAction::Move { from, to }) = board_action(ui, "arena_board") {
                self.state.handle_move(from, to, self.ws);
            }
        })
        .response
    }
}
