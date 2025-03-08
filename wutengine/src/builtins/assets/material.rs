use std::sync::{Arc, OnceLock};

use wutengine_graphics::color::Color;
use wutengine_graphics::material::{MaterialData, MaterialParameter};
use wutengine_graphics::renderer::{RendererMaterialId, WutEngineRenderer};
use wutengine_graphics::shader::ShaderId;

use crate::asset::Asset;

/// A material asset, describing how to render
/// a mesh. Works together with the [super::Mesh] asset to make
/// an entity renderable.
#[derive(Debug, Clone)]
pub struct Material(pub(crate) Arc<RawMaterial>);

/// The raw internal material data for a [Material] asset
#[derive(Debug)]
pub(crate) struct RawMaterial {
    renderer_id: OnceLock<RendererMaterialId>,

    /// The raw data assigned to this material
    pub(crate) data: MaterialData,
}

impl Clone for RawMaterial {
    fn clone(&self) -> Self {
        Self {
            renderer_id: OnceLock::new(),
            data: self.data.clone(),
        }
    }
}

impl RawMaterial {
    /// Returns the renderer ID for this material, initializing it and uploading the data if no ID was assigned yet
    pub(crate) fn get_renderer_id_or_init(
        &self,
        renderer: &mut impl WutEngineRenderer,
    ) -> RendererMaterialId {
        *self.renderer_id.get_or_init(|| {
            let id = renderer.create_material();
            renderer.update_material(id, &self.data);
            id
        })
    }
}

impl Material {
    /// Creates a new [Material] asset.
    pub fn new() -> Self {
        Self(Arc::new(RawMaterial {
            renderer_id: OnceLock::new(),
            data: MaterialData::default(),
        }))
    }

    /// Sets the shader of this material to a new value
    pub fn set_shader(&mut self, shader: Option<ShaderId>) {
        self.get_raw_mut_cloned().data.shader = shader;
    }

    /// Sets a color value on this material to a new value
    pub fn set_color(&mut self, name: impl Into<String>, color: Color) {
        self.get_raw_mut_cloned()
            .data
            .parameters
            .insert(name.into(), MaterialParameter::Color(color));
    }
}

/// Private utilities
impl Material {
    #[inline(always)]
    fn get_raw_mut_cloned(&mut self) -> &mut RawMaterial {
        Arc::make_mut(&mut self.0)
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}

impl Asset for Material {}
