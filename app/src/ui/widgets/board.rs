use crate::utils::images::Images;
use checkmade_core::game::set::PieceSet;
use checkmade_core::game::visuals::BoardVisuals;
use checkmade_core::giga_chess::prelude::{Color, Piece, Square};
use egui::{Color32, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardAction {
    Move { from: Square, to: Square },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum BoardInteraction {
    #[default]
    Idle,
    Selected(Square),
    Dragging(Square),
}

pub struct BoardWidget<'a> {
    images: &'a mut Images,
    vis: &'a BoardVisuals,
    id: Id,
    size: f32,
    can_move: bool,
    move_restriction: Option<Color>,
    light_color: Color32,
    dark_color: Color32,
    highlight_color: Color32,
    threat_target_color: Color32,
    threat_source_color: Color32,
}

impl<'a> BoardWidget<'a> {
    pub fn new(images: &'a mut Images, vis: &'a BoardVisuals, id: impl Into<Id>) -> Self {
        Self {
            images,
            vis,
            id: id.into(),
            size: 100.0,
            can_move: false,
            move_restriction: None,
            light_color: Color32::from_rgb(240, 217, 181),
            dark_color: Color32::from_rgb(181, 136, 99),
            highlight_color: Color32::from_rgba_premultiplied(145, 130, 50, 75),
            threat_target_color: Color32::from_rgba_premultiplied(210, 45, 35, 200),
            threat_source_color: Color32::from_rgba_premultiplied(200, 50, 40, 140),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn can_move(mut self, can_move: bool) -> Self {
        self.can_move = can_move;
        self
    }

    pub fn move_restriction(mut self, restriction: Option<Color>) -> Self {
        self.move_restriction = restriction;
        self
    }

    pub fn light_color(mut self, color: Color32) -> Self {
        self.light_color = color;
        self
    }

    pub fn dark_color(mut self, color: Color32) -> Self {
        self.dark_color = color;
        self
    }

    pub fn highlight_color(mut self, color: Color32) -> Self {
        self.highlight_color = color;
        self
    }

    pub fn threat_target_color(mut self, color: Color32) -> Self {
        self.threat_target_color = color;
        self
    }

    pub fn threat_source_color(mut self, color: Color32) -> Self {
        self.threat_source_color = color;
        self
    }

    fn interaction_id(&self) -> Id {
        self.id.with("interaction")
    }

    fn action_id(&self) -> Id {
        self.id.with("action")
    }

    fn is_flipped(&self) -> bool {
        self.vis.perspective == Color::Black
    }

    fn pos_to_square(&self, pos: Pos2, board_rect: &Rect) -> Option<Square> {
        let sq = board_rect.width() / 8.0;
        let local = pos - board_rect.min;
        let file = (local.x / sq) as i8;
        let rank = (local.y / sq) as i8;

        if !(0..8).contains(&file) || !(0..8).contains(&rank) {
            return None;
        }

        let (file, rank) = if self.is_flipped() {
            (7 - file, rank)
        } else {
            (file, 7 - rank)
        };

        Some(Square::from_file_rank(file as u8 + 1, rank as u8 + 1))
    }

    fn square_to_screen(&self, sq: Square, board_rect: &Rect) -> Pos2 {
        let cell = board_rect.width() / 8.0;
        let (file, rank) = if self.is_flipped() {
            (8 - sq.file() as i8, sq.rank() as i8 - 1)
        } else {
            (sq.file() as i8 - 1, 8 - sq.rank() as i8)
        };
        Pos2::new(
            board_rect.min.x + file as f32 * cell + cell * 0.5,
            board_rect.min.y + rank as f32 * cell + cell * 0.5,
        )
    }

    fn piece_at_square(&self, sq: Square) -> Option<(Piece, Color)> {
        self.vis.pieces.iter().find_map(|pv| {
            let pv_sq: Square = pv.coords.into();
            if pv_sq == sq {
                Some((pv.piece, pv.color))
            } else {
                None
            }
        })
    }

    fn can_interact_with(&self, sq: Square) -> bool {
        if !self.can_move {
            return false;
        }
        match (self.piece_at_square(sq), self.move_restriction) {
            (Some((_, piece_color)), Some(allowed)) => piece_color == allowed,
            (Some(_), None) => true,
            (None, _) => false,
        }
    }

    fn process_input(
        &self,
        ui: &Ui,
        response: &Response,
        board_rect: &Rect,
    ) -> (BoardInteraction, Option<BoardAction>) {
        let mut state: BoardInteraction = ui
            .memory(|mem| mem.data.get_temp(self.interaction_id()))
            .unwrap_or_default();

        let mut action: Option<BoardAction> = None;

        let hover_sq = response
            .hover_pos()
            .and_then(|p| self.pos_to_square(p, board_rect));

        if response.drag_started()
            && let Some(sq) = hover_sq
            && self.can_interact_with(sq)
        {
            state = BoardInteraction::Dragging(sq);
        }

        if response.drag_stopped()
            && let BoardInteraction::Dragging(from) = state
            && let Some(to) = hover_sq
        {
            if to != from {
                action = Some(BoardAction::Move { from, to });
            }
            state = BoardInteraction::Idle;
        }

        if response.clicked() {
            match state {
                BoardInteraction::Idle => {
                    if let Some(sq) = hover_sq
                        && self.can_interact_with(sq)
                    {
                        state = BoardInteraction::Selected(sq);
                    }
                }
                BoardInteraction::Selected(from) => {
                    if let Some(to) = hover_sq {
                        if to == from {
                            state = BoardInteraction::Idle;
                        } else {
                            action = Some(BoardAction::Move { from, to });
                            state = BoardInteraction::Idle;
                        }
                    } else {
                        state = BoardInteraction::Idle;
                    }
                }
                _ => {}
            }
        }

        if response.secondary_clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            state = BoardInteraction::Idle;
        }

        (state, action)
    }

    fn paint_squares(&self, painter: &egui::Painter, board_rect: &Rect) {
        let sq = board_rect.width() / 8.0;
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let color = if (rank + file) % 2 == 0 {
                    self.light_color
                } else {
                    self.dark_color
                };
                let min = Pos2::new(
                    board_rect.min.x + file as f32 * sq,
                    board_rect.min.y + rank as f32 * sq,
                );
                painter.rect_filled(Rect::from_min_size(min, Vec2::splat(sq)), 0.0, color);
            }
        }

        painter.rect_stroke(
            *board_rect,
            0.0,
            Stroke::new(2.0, Color32::from_rgb(80, 60, 40)),
            StrokeKind::Middle,
        );
    }

    fn paint_highlight(
        &self,
        painter: &egui::Painter,
        board_rect: &Rect,
        square: Square,
        color: Color32,
    ) {
        let cell = board_rect.width() / 8.0;
        let center = self.square_to_screen(square, board_rect);
        let rect = Rect::from_center_size(center, Vec2::splat(cell));
        painter.rect_filled(rect, 0.0, color);
    }

    fn paint_pieces(
        &mut self,
        ui: &Ui,
        board_rect: &Rect,
        interaction: BoardInteraction,
        response: &Response,
    ) {
        let sq_size = board_rect.width() / 8.0;
        let painter = ui.painter();

        let dragging_sq = match interaction {
            BoardInteraction::Dragging(sq) => Some(sq),
            _ => None,
        };

        let board_half = 32768.0_f32;
        let board_center = board_rect.center();

        for pv in &self.vis.pieces {
            let pv_sq: Square = pv.coords.into();

            if dragging_sq == Some(pv_sq) {
                continue;
            }

            let nx = pv.coords.x as f32 / board_half;
            let ny = pv.coords.y as f32 / board_half;
            let (sx, sy) = if self.is_flipped() {
                (
                    board_center.x - nx * board_rect.width(),
                    board_center.y + ny * board_rect.height(),
                )
            } else {
                (
                    board_center.x + nx * board_rect.width(),
                    board_center.y - ny * board_rect.height(),
                )
            };

            let piece_size = sq_size * 0.8;
            let dest = Rect::from_center_size(Pos2::new(sx, sy), Vec2::splat(piece_size));
            let raster_size = sq_size * ui.ctx().pixels_per_point();
            if let Some(tex_id) = self.images.piece_texture(
                ui.ctx(),
                PieceSet::Regular,
                pv.piece,
                pv.color,
                raster_size,
            ) {
                painter.image(
                    tex_id,
                    dest,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }

        if let BoardInteraction::Dragging(from) = interaction
            && let Some(pointer) = response.interact_pointer_pos()
            && let Some(pv) = self
                .vis
                .pieces
                .iter()
                .find(|pv| Square::from(pv.coords) == from)
        {
            let piece_size = sq_size * 0.9;
            let dest = Rect::from_center_size(pointer, Vec2::splat(piece_size));
            let raster_size = sq_size * ui.ctx().pixels_per_point();
            if let Some(tex_id) = self.images.piece_texture(
                ui.ctx(),
                PieceSet::Regular,
                pv.piece,
                pv.color,
                raster_size,
            ) {
                painter.image(
                    tex_id,
                    dest,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
        }
    }

    fn paint_labels(&self, ui: &Ui, board_rect: &Rect) {
        let sq = board_rect.width() / 8.0;
        let font = egui::FontId::proportional(sq * 0.18);
        let painter = ui.painter();

        let files_top: [char; 8] = if self.is_flipped() {
            ['h', 'g', 'f', 'e', 'd', 'c', 'b', 'a']
        } else {
            ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']
        };
        let ranks_left: [char; 8] = if self.is_flipped() {
            ['1', '2', '3', '4', '5', '6', '7', '8']
        } else {
            ['8', '7', '6', '5', '4', '3', '2', '1']
        };

        let margin = sq * 0.08;

        for i in 0..8u8 {
            let file_label_color = if (7 + i) % 2 == 0 {
                self.dark_color
            } else {
                self.light_color
            };
            let file_pos = Pos2::new(
                board_rect.min.x + (i as f32 + 1.0) * sq - margin,
                board_rect.max.y - margin,
            );
            painter.text(
                file_pos,
                egui::Align2::RIGHT_BOTTOM,
                files_top[i as usize],
                font.clone(),
                file_label_color,
            );

            let rank_label_color = if (i) % 2 == 0 {
                self.dark_color
            } else {
                self.light_color
            };
            let rank_pos = Pos2::new(
                board_rect.min.x + margin,
                board_rect.min.y + i as f32 * sq + margin,
            );
            painter.text(
                rank_pos,
                egui::Align2::LEFT_TOP,
                ranks_left[i as usize],
                font.clone(),
                rank_label_color,
            );
        }
    }
}

impl<'a> egui::Widget for BoardWidget<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(self.size), Sense::click_and_drag());

        let (interaction, action) = self.process_input(ui, &response, &rect);

        ui.memory_mut(|mem| mem.data.insert_temp(self.interaction_id(), interaction));

        ui.memory_mut(|mem| {
            if let Some(act) = action {
                mem.data.insert_temp(self.action_id(), act);
            } else {
                mem.data.remove::<BoardAction>(self.action_id());
            }
        });

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);

            self.paint_squares(&painter, &rect);

            match interaction {
                BoardInteraction::Selected(sq) | BoardInteraction::Dragging(sq) => {
                    self.paint_highlight(&painter, &rect, sq, self.highlight_color);
                }
                _ => {}
            }

            if let Some(from) = self.vis.last_move_from
                && !self.vis.threat_sources.contains(&from)
                && !self.vis.threat_targets.contains(&from)
            {
                self.paint_highlight(&painter, &rect, from, self.highlight_color);
            }

            if let Some(to) = self.vis.last_move_to
                && !self.vis.threat_sources.contains(&to)
                && !self.vis.threat_targets.contains(&to)
            {
                self.paint_highlight(&painter, &rect, to, self.highlight_color);
            }

            for sq in &self.vis.threat_sources {
                self.paint_highlight(&painter, &rect, *sq, self.threat_source_color);
            }

            for sq in &self.vis.threat_targets {
                self.paint_highlight(&painter, &rect, *sq, self.threat_target_color);
            }

            self.paint_pieces(ui, &rect, interaction, &response);
            self.paint_labels(ui, &rect);
        }

        response
    }
}

pub fn board_action(ui: &Ui, id: impl Into<Id>) -> Option<BoardAction> {
    let id: Id = id.into();
    ui.memory(|mem| mem.data.get_temp(id.with("action")))
}
