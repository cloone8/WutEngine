use std::rc::Rc;

use glam::Vec3;
use wutengine_core::Component;
use wutengine_graphics::mesh::MeshData;

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

impl Component for Mesh {}
