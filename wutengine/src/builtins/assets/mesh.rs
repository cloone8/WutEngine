use std::sync::Arc;

use glam::Vec3;
use wutengine_graphics::mesh::MeshData;

use crate::asset::Asset;

/// A renderable mesh. Works together with the [super::Material] component
/// to enable an entity to be rendered.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// The actual mesh data.
    /// Allows multiple meshes to use the same data transparently
    pub(crate) data: Arc<MeshData>,
}

impl Mesh {
    /// Returns the read-only vertex positions of this [Mesh]
    pub fn get_vertices(&self) -> &[Vec3] {
        &self.data.positions
    }

    /// Creates a new [Mesh]
    pub fn new(data: MeshData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }
}

impl Asset for Mesh {}
