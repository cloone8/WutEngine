use std::sync::{Arc, RwLock};

use wutengine_graphics::image::DynamicImage;
use wutengine_graphics::renderer::{RendererTexture2DId, WutEngineRenderer};
use wutengine_graphics::texture::{TextureData, TextureFiltering, TextureWrapping};

use crate::asset::Asset;

/// A texture asset
#[derive(Debug, Clone)]
pub struct Texture(pub(crate) Arc<RwLock<RawTexture>>);

impl Texture {
    /// Creates a new texture asset.
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(RawTexture {
            renderer_id: RendererTexture2DId::new(),
            dirty: true,
            data: TextureData::default(),
        })))
    }

    /// Creates a private clone of this texture and sets its image data
    pub fn set_image(&mut self, image: impl Into<DynamicImage>) {
        let raw = self.get_raw_mut_cloned();

        raw.data.imagedata = image.into();
        raw.dirty = true;
    }

    /// Sets the image data of this texture for all its current users
    pub fn set_image_shared(&mut self, image: impl Into<DynamicImage>) {
        let mut raw = self.0.write().unwrap();
        raw.data.imagedata = image.into();
        raw.dirty = true;
    }

    /// Creates a private clone of this texture and sets the texture filtering method
    pub fn set_filter(&mut self, filter: TextureFiltering) {
        let raw = self.get_raw_mut_cloned();

        raw.data.filtering = filter;
        raw.dirty = true;
    }

    /// Sets the texture filtering method for all current users of this texture
    pub fn set_filter_shared(&mut self, filter: TextureFiltering) {
        let mut raw = self.0.write().unwrap();

        raw.data.filtering = filter;
        raw.dirty = true;
    }

    /// Creates a private clone of this texture and sets the texture wrapping method
    pub fn set_wrapping(&mut self, wrapping: TextureWrapping) {
        let raw = self.get_raw_mut_cloned();

        raw.data.wrapping = wrapping;
        raw.dirty = true;
    }

    /// Sets the texture wrapping method for all current users of this texture
    pub fn set_wrapping_shared(&mut self, wrapping: TextureWrapping) {
        let mut raw = self.0.write().unwrap();

        raw.data.wrapping = wrapping;
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
    pub(crate) renderer_id: RendererTexture2DId,

    dirty: bool,

    /// The raw data assigned to this texture
    pub(crate) data: TextureData,
}

impl Clone for RawTexture {
    fn clone(&self) -> Self {
        Self {
            renderer_id: RendererTexture2DId::new(),
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

        renderer.update_texture2d(self.renderer_id, &self.data);

        self.dirty = false;
    }
}
