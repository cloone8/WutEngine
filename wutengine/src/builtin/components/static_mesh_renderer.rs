use serde::{Deserialize, Serialize};
use wutengine_asset::AssetHandle;
use wutengine_graphics::material::Material;
use wutengine_graphics::mesh::Mesh;

use crate::component::{Component, Renderer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticMeshRenderer {
    mesh: Option<AssetHandle<Mesh>>,
    material: Option<AssetHandle<Material>>,
}

impl Default for StaticMeshRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticMeshRenderer {
    pub fn new() -> Self {
        Self {
            mesh: None,
            material: None,
        }
    }

    pub fn set_mesh(&mut self, mesh: Option<impl Into<AssetHandle<Mesh>>>) {
        self.mesh = mesh.map(|m| m.into());
    }

    pub fn set_material(&mut self, material: Option<impl Into<AssetHandle<Material>>>) {
        self.material = material.map(|m| m.into());
    }
}

impl Component for StaticMeshRenderer {
    fn as_renderer(&mut self) -> Option<&mut dyn Renderer> {
        Some(self)
    }
    // fn on_render(&mut self, _context: crate::prelude::ComponentContext) {
    //     log::debug!("StaticMeshRenderer on_render");

    //     let (mesh, material) = match (&mut self.mesh, &mut self.material) {
    //         (Some(mesh), Some(mat)) => (mesh, mat),
    //         // Nothing to do
    //         _ => return,
    //     };
    // }
}

impl Renderer for StaticMeshRenderer {}
