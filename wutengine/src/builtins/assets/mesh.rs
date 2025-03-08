use std::sync::{Arc, OnceLock};

use glam::Vec3;
use wutengine_graphics::mesh::MeshData;
use wutengine_graphics::renderer::{RendererMeshId, WutEngineRenderer};

use crate::asset::Asset;

/// A renderable mesh. Works together with a [super::Material] asset
/// to enable an entity to be rendered.
#[derive(Debug, Clone)]
pub struct Mesh(pub(crate) Arc<RawMesh>);

/// The raw internal mesh data for a [Mesh] asset
#[derive(Debug)]
pub(crate) struct RawMesh {
    renderer_id: OnceLock<RendererMeshId>,

    /// The actual mesh data.
    /// Allows multiple meshes to use the same data transparently
    pub(crate) data: MeshData,
}

impl Clone for RawMesh {
    fn clone(&self) -> Self {
        Self {
            renderer_id: OnceLock::new(),
            data: self.data.clone(),
        }
    }
}

impl RawMesh {
    /// Returns the renderer ID for this mesh, initializing it and uploading the data if no ID was assigned yet
    pub(crate) fn get_renderer_id_or_init(
        &self,
        renderer: &mut impl WutEngineRenderer,
    ) -> RendererMeshId {
        *self.renderer_id.get_or_init(|| {
            let id = renderer.create_mesh();
            renderer.update_mesh(id, &self.data);
            id
        })
    }
}

impl Mesh {
    /// Returns the read-only vertex positions of this [Mesh]
    pub fn get_vertices(&self) -> &[Vec3] {
        &self.0.data.positions
    }

    /// Creates a new [Mesh]
    pub fn new(data: MeshData) -> Self {
        Self(Arc::new(RawMesh {
            renderer_id: OnceLock::new(),
            data,
        }))
    }
}

impl Asset for Mesh {}
