use crate::utils::image_source::ImageSource;
use egui::{TextureHandle, TextureOptions};
use std::collections::HashMap;
use std::hash::Hash;

pub struct ImageCache<K: Hash + Eq> {
    sources: HashMap<K, ImageSource>,
    textures: HashMap<K, (u32, TextureHandle)>,
}

impl<K: Hash + Eq + Clone + std::fmt::Debug> ImageCache<K> {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, source: ImageSource) {
        self.textures.remove(&key);
        self.sources.insert(key, source);
    }

    pub fn get(&mut self, ctx: &egui::Context, key: &K, size_px: f32) -> Option<&TextureHandle> {
        let target_size = ((size_px.ceil() as u32).div_ceil(16) * 16).max(16);

        let needs_raster = match self.textures.get(key) {
            Some((cached, _)) => *cached != target_size,
            None => true,
        };

        if needs_raster {
            let source = self.sources.get(key)?;
            let image = source.rasterize(target_size);
            let handle = ctx.load_texture(format!("{key:?}"), image, TextureOptions::LINEAR);
            self.textures.insert(key.clone(), (target_size, handle));
        }

        self.textures.get(key).map(|(_, h)| h)
    }
}
