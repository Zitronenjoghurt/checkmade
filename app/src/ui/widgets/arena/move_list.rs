use crate::i18n::Translatable;
use crate::ui::icons;
use checkmade_core::lingo::Lingo::NoMoveHistory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveListEvent {
    Move(usize),
    Back,
    Forward,
    Start,
    Present,
}

pub struct MoveListWidget<'a> {
    san_history: &'a [String],
    view_index: Option<usize>,
}

impl<'a> MoveListWidget<'a> {
    pub fn new(san_history: &'a [String], view_index: Option<usize>) -> Self {
        Self {
            san_history,
            view_index,
        }
    }

    fn active_index(&self) -> Option<usize> {
        match self.view_index {
            Some(idx) => Some(idx),
            None if !self.san_history.is_empty() => Some(self.san_history.len()),
            None => None,
        }
    }
}

impl MoveListWidget<'_> {
    pub fn show(&self, ui: &mut egui::Ui) -> Option<MoveListEvent> {
        let mut event = None;
        let active = self.active_index();

        let total_rounds = ((self.san_history.len() + 1) / 2).max(1);
        let body_height = ui.text_style_height(&egui::TextStyle::Body);
        let num_width = ui.fonts_mut(|f| {
            f.layout_no_wrap(
                format!("{}.", total_rounds),
                egui::FontId::monospace(body_height),
                egui::Color32::WHITE,
            )
            .size()
            .x
        }) + 8.0;

        let move_col_width = 56.0;
        let row_height = ui.spacing().interact_size.y;
        let list_width = num_width + move_col_width * 2.0;

        let nav_height = row_height + ui.spacing().item_spacing.y;
        let scroll_height = (ui.available_height() - nav_height).max(row_height);

        ui.horizontal(|ui| {
            ui.set_max_width(list_width);
            let btn_width = (list_width - ui.spacing().item_spacing.x * 3.0) / 4.0;

            if ui
                .add_sized(
                    [btn_width, row_height],
                    egui::Button::new(icons::CARET_LINE_LEFT),
                )
                .clicked()
            {
                event = Some(MoveListEvent::Start);
            }
            if ui
                .add_sized(
                    [btn_width, row_height],
                    egui::Button::new(icons::CARET_LEFT),
                )
                .clicked()
            {
                event = Some(MoveListEvent::Back);
            }
            if ui
                .add_sized(
                    [btn_width, row_height],
                    egui::Button::new(icons::CARET_RIGHT),
                )
                .clicked()
            {
                event = Some(MoveListEvent::Forward);
            }
            if ui
                .add_sized(
                    [btn_width, row_height],
                    egui::Button::new(icons::CARET_LINE_RIGHT),
                )
                .clicked()
            {
                event = Some(MoveListEvent::Present);
            }
        });

        egui::ScrollArea::vertical()
            .stick_to_bottom(self.view_index.is_none())
            .max_height(scroll_height)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    if self.san_history.is_empty() {
                        ui.small(NoMoveHistory.t());
                    }

                    for (i, chunk) in self.san_history.chunks(2).enumerate() {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;

                            ui.add_sized(
                                [num_width, row_height],
                                egui::Label::new(
                                    egui::RichText::new(format!("{}.", i + 1))
                                        .monospace()
                                        .weak(),
                                ),
                            );

                            let w_idx = i * 2 + 1;
                            if ui
                                .add_sized(
                                    [move_col_width, row_height],
                                    egui::Button::selectable(active == Some(w_idx), &chunk[0]),
                                )
                                .clicked()
                            {
                                event = Some(MoveListEvent::Move(w_idx));
                            }

                            if let Some(black_san) = chunk.get(1) {
                                let b_idx = i * 2 + 2;
                                if ui
                                    .add_sized(
                                        [move_col_width, row_height],
                                        egui::Button::selectable(active == Some(b_idx), black_san),
                                    )
                                    .clicked()
                                {
                                    event = Some(MoveListEvent::Move(b_idx));
                                }
                            }
                        });
                    }
                });
            });

        event
    }
}
