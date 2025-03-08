use std::sync::{Arc, OnceLock};

use wutengine_graphics::material::MaterialData;
use wutengine_graphics::renderer::{RendererMaterialId, WutEngineRenderer};

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
    pub fn new(data: MaterialData) -> Self {
        Self(Arc::new(RawMaterial {
            renderer_id: OnceLock::new(),
            data,
        }))
    }
}

impl Asset for Material {}
