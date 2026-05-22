use crate::utils::images::Images;
use checkmade_core::game::set::PieceSet;
use checkmade_core::game::visuals::BoardVisuals;
use checkmade_core::giga_chess::prelude::Color;
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

pub struct BoardWidget<'a> {
    images: &'a mut Images,
    vis: &'a BoardVisuals,
    size: f32,
    light_color: Color32,
    dark_color: Color32,
}

impl<'a> BoardWidget<'a> {
    pub fn new(images: &'a mut Images, vis: &'a BoardVisuals) -> Self {
        Self {
            images,
            vis,
            size: 100.0,
            light_color: Color32::from_rgb(240, 217, 181),
            dark_color: Color32::from_rgb(181, 136, 99),
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
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

    fn is_flipped(&self) -> bool {
        self.vis.perspective == Color::Black
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

    fn paint_pieces(&mut self, ui: &Ui, board_rect: &Rect) {
        let sq = board_rect.width() / 8.0;
        let painter = ui.painter();

        let board_half = 32768.0_f32;
        let board_center = board_rect.center();

        for pv in &self.vis.pieces {
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

            let piece_size = sq * 0.8;
            let dest = Rect::from_center_size(Pos2::new(sx, sy), Vec2::splat(piece_size));
            let raster_size = sq * ui.ctx().pixels_per_point();
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

        if ui.is_rect_visible(rect) {
            let painter = ui.painter_at(rect);

            self.paint_squares(&painter, &rect);
            self.paint_pieces(ui, &rect);
            self.paint_labels(ui, &rect);
        }

        response
    }
}
