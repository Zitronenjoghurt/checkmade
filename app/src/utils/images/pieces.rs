use crate::utils::image_cache::ImageCache;
use crate::utils::image_source::ImageSource;
use checkmade_core::game::set::PieceSet;
use checkmade_core::giga_chess::prelude::{Color, Piece};

struct PieceSetAssets {
    bb: &'static [u8],
    bk: &'static [u8],
    bn: &'static [u8],
    bp: &'static [u8],
    bq: &'static [u8],
    br: &'static [u8],
    wb: &'static [u8],
    wk: &'static [u8],
    wn: &'static [u8],
    wp: &'static [u8],
    wq: &'static [u8],
    wr: &'static [u8],
}

impl PieceSetAssets {
    fn get(&self, piece: Piece, color: Color) -> &'static [u8] {
        match (color, piece) {
            (Color::Black, Piece::Bishop) => self.bb,
            (Color::Black, Piece::King) => self.bk,
            (Color::Black, Piece::Knight) => self.bn,
            (Color::Black, Piece::Pawn) => self.bp,
            (Color::Black, Piece::Queen) => self.bq,
            (Color::Black, Piece::Rook) => self.br,
            (Color::White, Piece::Bishop) => self.wb,
            (Color::White, Piece::King) => self.wk,
            (Color::White, Piece::Knight) => self.wn,
            (Color::White, Piece::Pawn) => self.wp,
            (Color::White, Piece::Queen) => self.wq,
            (Color::White, Piece::Rook) => self.wr,
        }
    }
}

static REGULAR: PieceSetAssets = PieceSetAssets {
    bb: include_bytes!("../../../assets/sets/regular/bb.svg"),
    bk: include_bytes!("../../../assets/sets/regular/bk.svg"),
    bn: include_bytes!("../../../assets/sets/regular/bn.svg"),
    bp: include_bytes!("../../../assets/sets/regular/bp.svg"),
    bq: include_bytes!("../../../assets/sets/regular/bq.svg"),
    br: include_bytes!("../../../assets/sets/regular/br.svg"),
    wb: include_bytes!("../../../assets/sets/regular/wb.svg"),
    wk: include_bytes!("../../../assets/sets/regular/wk.svg"),
    wn: include_bytes!("../../../assets/sets/regular/wn.svg"),
    wp: include_bytes!("../../../assets/sets/regular/wp.svg"),
    wq: include_bytes!("../../../assets/sets/regular/wq.svg"),
    wr: include_bytes!("../../../assets/sets/regular/wr.svg"),
};

fn get_assets(set: PieceSet) -> &'static PieceSetAssets {
    match set {
        PieceSet::Regular => &REGULAR,
    }
}

pub fn build() -> ImageCache<(PieceSet, Piece, Color)> {
    let mut cache = ImageCache::new();
    for set in <PieceSet as strum::IntoEnumIterator>::iter() {
        let assets = get_assets(set);
        for piece in Piece::ALL {
            for color in Color::ALL {
                cache.insert(
                    (set, piece, color),
                    ImageSource::from_bytes(assets.get(piece, color)),
                );
            }
        }
    }
    cache
}
