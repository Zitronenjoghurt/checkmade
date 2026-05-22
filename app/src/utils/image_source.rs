use eframe::epaint::{Color32, ColorImage};
use resvg::tiny_skia::Pixmap;
use resvg::{tiny_skia, usvg};

pub enum ImageSource {
    Svg(usvg::Tree),
    Raster {
        pixels: Vec<u8>,
        width: u32,
        height: u32,
    },
}

impl ImageSource {
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.starts_with(&[0x89, b'P', b'N', b'G']) {
            Self::from_png(data)
        } else {
            Self::from_svg(data)
        }
    }

    pub fn from_svg(data: &[u8]) -> Self {
        let tree =
            usvg::Tree::from_data(data, &usvg::Options::default()).expect("invalid SVG data");
        Self::Svg(tree)
    }

    pub fn from_png(data: &[u8]) -> Self {
        let image = image::load_from_memory_with_format(data, image::ImageFormat::Png)
            .expect("invalid PNG data")
            .into_rgba8();
        let (w, h) = image.dimensions();
        Self::Raster {
            pixels: image.into_raw(),
            width: w,
            height: h,
        }
    }

    pub fn rasterize(&self, size_px: u32) -> ColorImage {
        match self {
            Self::Svg(tree) => rasterize_svg(tree, size_px),
            Self::Raster {
                pixels,
                width,
                height,
            } => rasterize_rgba(pixels, *width, *height, size_px),
        }
    }
}

fn rasterize_svg(tree: &usvg::Tree, size_px: u32) -> ColorImage {
    let mut pixmap = Pixmap::new(size_px, size_px).expect("zero-sized pixmap");

    let svg_size = tree.size();

    let scale = (size_px as f32 / svg_size.width()).min(size_px as f32 / svg_size.height());
    let scaled_width = svg_size.width() * scale;
    let scaled_height = svg_size.height() * scale;
    let dx = (size_px as f32 - scaled_width) / 2.0;
    let dy = (size_px as f32 - scaled_height) / 2.0;

    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(dx, dy);

    resvg::render(tree, transform, &mut pixmap.as_mut());

    pixmap_to_color_image(pixmap)
}

fn rasterize_rgba(pixels: &[u8], w: u32, h: u32, size_px: u32) -> ColorImage {
    let src =
        image::RgbaImage::from_raw(w, h, pixels.to_vec()).expect("pixel buffer size mismatch");

    let resized = image::imageops::resize(
        &src,
        size_px,
        size_px,
        image::imageops::FilterType::Lanczos3,
    );

    let colors: Vec<Color32> = resized
        .pixels()
        .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    ColorImage {
        size: [size_px as usize, size_px as usize],
        source_size: Default::default(),
        pixels: colors,
    }
}

fn pixmap_to_color_image(pixmap: Pixmap) -> ColorImage {
    let colors: Vec<Color32> = pixmap
        .data()
        .chunks_exact(4)
        .map(|c| Color32::from_rgba_premultiplied(c[0], c[1], c[2], c[3]))
        .collect();

    ColorImage {
        size: [pixmap.width() as usize, pixmap.height() as usize],
        source_size: Default::default(),
        pixels: colors,
    }
}
