use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use wutengine_graphics::color::Color;
use wutengine_graphics::material::{MaterialData, MaterialParameter};
use wutengine_graphics::renderer::{RendererMaterialId, WutEngineRenderer};
use wutengine_graphics::shader::ShaderVariantId;

use crate::asset::Asset;

use super::Texture;

/// A material asset, describing how to render
/// a mesh. Works together with the [super::Mesh] asset to make
/// an entity renderable.
#[derive(Debug, Clone)]
pub struct Material(pub(crate) Arc<RwLock<RawMaterial>>);

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}

impl Asset for Material {}

impl Material {
    /// Creates a new [Material] asset.
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(RawMaterial {
            renderer_id: RendererMaterialId::new(),
            dirty: true,
            used_textures: HashMap::default(),
            data: MaterialData::default(),
        })))
    }

    /// Sets the shader of this material to a new value
    pub fn set_shader(&mut self, shader: Option<ShaderVariantId>) {
        let raw = self.get_raw_mut_cloned();
        raw.data.shader = shader;
        raw.dirty = true;
    }

    /// Sets a color value on this material to a new value
    pub fn set_color(&mut self, name: impl Into<String>, color: Color) {
        let raw = self.get_raw_mut_cloned();

        raw.data
            .parameters
            .insert(name.into(), MaterialParameter::Vec4(color.into()));
        raw.dirty = true;
    }

    /// Sets a u32 value on this material to a new value
    pub fn set_u32(&mut self, name: impl Into<String>, n: u32) {
        let raw = self.get_raw_mut_cloned();

        raw.data
            .parameters
            .insert(name.into(), MaterialParameter::U32(n));
        raw.dirty = true;
    }

    /// Sets a texture value on this material to a new value
    pub fn set_texture(&mut self, name: impl Into<String>, texture: Texture) {
        let raw = self.get_raw_mut_cloned();
        let name = name.into();

        raw.data.parameters.insert(
            name.clone(),
            MaterialParameter::Texture2D(texture.0.read().unwrap().renderer_id),
        );

        raw.used_textures.insert(name, texture);
        raw.dirty = true;
    }

    /// Unsets any value on this material
    pub fn unset(&mut self, name: impl AsRef<str>) {
        let raw = self.get_raw_mut_cloned();

        raw.data.parameters.remove(name.as_ref());
        raw.used_textures.remove(name.as_ref());
        raw.dirty = true;
    }
}

/// Private utilities
impl Material {
    fn get_raw_mut_cloned(&mut self) -> &mut RawMaterial {
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

/// The raw internal material data for a [Material] asset
#[derive(Debug)]
pub(crate) struct RawMaterial {
    /// The renderer ID for this material
    pub(crate) renderer_id: RendererMaterialId,

    dirty: bool,

    /// The textures currently in use by this material, keyed by the parameter name
    used_textures: HashMap<String, Texture>,

    /// The raw data assigned to this material
    pub(crate) data: MaterialData,
}

impl Clone for RawMaterial {
    fn clone(&self) -> Self {
        let new_id = RendererMaterialId::new();

        Self {
            renderer_id: new_id,
            dirty: true,
            used_textures: self.used_textures.clone(),
            data: self.data.clone(),
        }
    }
}

impl RawMaterial {
    /// Flushes the changes on this material to the given renderer, if required
    pub(crate) fn flush(&mut self, renderer: &mut impl WutEngineRenderer) {
        if !self.dirty {
            return;
        }

        for texture in self.used_textures.values() {
            texture.0.write().unwrap().flush(renderer);
        }

        renderer.update_material(self.renderer_id, &self.data);

        self.dirty = false;
    }

    /// Flushes the given material and returns its ID
    pub(crate) fn flush_and_get_id(
        this: &Arc<RwLock<Self>>,
        renderer: &mut impl WutEngineRenderer,
    ) -> RendererMaterialId {
        let mut locked = this.write().unwrap();

        locked.flush(renderer);

        locked.renderer_id
    }
}
