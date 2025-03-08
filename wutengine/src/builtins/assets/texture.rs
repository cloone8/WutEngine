use std::sync::{Arc, OnceLock};

use wutengine_graphics::renderer::{RendererTextureId, WutEngineRenderer};
use wutengine_graphics::texture::TextureData;

use crate::asset::Asset;

/// A texture asset
#[derive(Debug, Clone)]
pub struct Texture(pub(crate) Arc<RawTexture>);

/// The raw internal texture data for a [Texture] asset
#[derive(Debug)]
pub(crate) struct RawTexture {
    renderer_id: OnceLock<RendererTextureId>,

    /// The raw data assigned to this texture
    pub(crate) data: TextureData,
}

impl Clone for RawTexture {
    fn clone(&self) -> Self {
        Self {
            renderer_id: OnceLock::new(),
            data: self.data.clone(),
        }
    }
}

impl RawTexture {
    /// Returns the renderer ID for this texture, initializing it and uploading the data if no ID was assigned yet
    pub(crate) fn get_renderer_id_or_init(
        &self,
        renderer: &mut impl WutEngineRenderer,
    ) -> RendererTextureId {
        *self.renderer_id.get_or_init(|| {
            let id = renderer.create_texture();
            renderer.update_texture(id, &self.data);
            id
        })
    }
}

impl Texture {
    /// Creates a new texture asset.
    pub fn new() -> Self {
        Self(Arc::new(RawTexture {
            renderer_id: OnceLock::new(),
            data: TextureData::default(),
        }))
    }
}

impl Default for Texture {
    fn default() -> Self {
        Self::new()
    }
}

impl Asset for Texture {}
