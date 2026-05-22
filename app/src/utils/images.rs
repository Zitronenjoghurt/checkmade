use crate::utils::image_cache::ImageCache;
use checkmade_core::game::set::PieceSet;
use checkmade_core::giga_chess::prelude::{Color, Piece};
use egui::Vec2;

mod pieces;

pub struct Images {
    pieces: ImageCache<(PieceSet, Piece, Color)>,
}

impl Default for Images {
    fn default() -> Self {
        Self {
            pieces: pieces::build(),
        }
    }
}

impl Images {
    pub fn piece_widget(
        &mut self,
        ctx: &egui::Context,
        set: PieceSet,
        piece: Piece,
        color: Color,
        size: f32,
    ) -> Option<egui::Image<'static>> {
        let handle = self.pieces.get(ctx, &(set, piece, color), size)?;
        Some(egui::Image::from_texture(egui::load::SizedTexture::new(
            handle.id(),
            Vec2::splat(size),
        )))
    }

    pub fn piece_texture(
        &mut self,
        ctx: &egui::Context,
        set: PieceSet,
        piece: Piece,
        color: Color,
        size: f32,
    ) -> Option<egui::TextureId> {
        Some(self.pieces.get(ctx, &(set, piece, color), size)?.id())
    }
}
