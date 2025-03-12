use std::sync::{Arc, RwLock};

use wutengine_graphics::image::DynamicImage;
use wutengine_graphics::renderer::{RendererTextureId, WutEngineRenderer};
use wutengine_graphics::texture::TextureData;

use crate::asset::Asset;

/// A texture asset
#[derive(Debug, Clone)]
pub struct Texture(pub(crate) Arc<RwLock<RawTexture>>);

impl Texture {
    /// Creates a new texture asset.
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(RawTexture {
            renderer_id: RendererTextureId::new(),
            dirty: true,
            data: TextureData::default(),
        })))
    }

    /// Sets the image data for this texture
    pub fn set_image(&mut self, image: impl Into<DynamicImage>) {
        let raw = self.get_raw_mut_cloned();

        raw.data.imagedata = image.into();
        raw.dirty = true;
    }
}

/// Private utilities
impl Texture {
    fn get_raw_mut_cloned(&mut self) -> &mut RawTexture {
        let is_unique = Arc::get_mut(&mut self.0).is_some();

        if !is_unique {
            let new_arc = {
                let cloned = self.0.read().unwrap().clone();

                Arc::new(RwLock::new(cloned))
            };

            self.0 = new_arc;
        }

        Arc::get_mut(&mut self.0)
            .expect("Should be unique")
            .get_mut()
            .unwrap()
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::new()
    }
}

impl Asset for Texture {}

/// The raw internal texture data for a [Texture] asset
#[derive(Debug)]
pub(crate) struct RawTexture {
    /// The renderer ID for this texture
    pub(crate) renderer_id: RendererTextureId,

    dirty: bool,

    /// The raw data assigned to this texture
    pub(crate) data: TextureData,
}

impl Clone for RawTexture {
    fn clone(&self) -> Self {
        Self {
            renderer_id: RendererTextureId::new(),
            dirty: true,
            data: self.data.clone(),
        }
    }
}

impl RawTexture {
    /// Flushes the changes on this texture to the given renderer, if needed
    pub(crate) fn flush(&mut self, renderer: &mut impl WutEngineRenderer) {
        if !self.dirty {
            return;
        }

        renderer.update_texture(self.renderer_id, &self.data);

        self.dirty = false;
    }
}
