use std::rc::Rc;

use glam::Vec3;
use wutengine_core::{Component, ComponentTypeId, DynComponent};
use wutengine_graphics::mesh::MeshData;

use super::ID_MESH;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub(crate) data: Rc<MeshData>,
}

impl Mesh {
    pub fn get_vertices(&self) -> &[Vec3] {
        &self.data.positions
    }

    pub fn new(data: MeshData) -> Self {
        Self {
            data: Rc::new(data),
        }
    }
}

impl DynComponent for Mesh {
    fn get_dyn_component_id(&self) -> ComponentTypeId {
        Self::COMPONENT_ID
    }
}

impl Component for Mesh {
    const COMPONENT_ID: ComponentTypeId = ID_MESH;
}
