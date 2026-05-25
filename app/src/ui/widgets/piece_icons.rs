use checkmade_core::giga_chess::prelude::{Color, Piece};
use egui::{Response, Ui};

pub struct PieceIcons<'a> {
    pieces: &'a [Piece],
    color: Color,
    images: &'a mut crate::utils::images::Images,
    overlap: f32, // 0.0 = no overlap, e.g. 4.0 = 4px overlap
}

impl<'a> PieceIcons<'a> {
    pub fn new(
        pieces: &'a [Piece],
        color: Color,
        images: &'a mut crate::utils::images::Images,
    ) -> Self {
        Self {
            pieces,
            color,
            images,
            overlap: 0.0,
        }
    }

    pub fn overlap(mut self, overlap: f32) -> Self {
        self.overlap = overlap;
        self
    }
}

impl egui::Widget for PieceIcons<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut pieces = self.pieces.to_vec();
        pieces.sort_unstable();

        ui.horizontal(|ui| {
            if self.overlap > 0.0 {
                ui.spacing_mut().item_spacing.x = -self.overlap;
            }

            for piece in pieces {
                if let Some(image) = self.images.piece_icon(ui.ctx(), piece, self.color, 16.0) {
                    ui.add(image);
                }
            }
        })
        .response
    }
}
