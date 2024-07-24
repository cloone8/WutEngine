use std::rc::Rc;

use rand::{rngs::SmallRng, Rng, SeedableRng};
use wutengine_core::{
    component::{Component, ComponentTypeId, DynComponent},
    math::Vec3,
    renderer::MeshData,
};

use super::ID_MESH;

#[derive(Debug)]
pub struct Mesh {
    /// Unique mesh identifier
    id: usize,
    pub(crate) data: Rc<MeshData>,
}

impl Clone for Mesh {
    fn clone(&self) -> Self {
        Self {
            id: gen_mesh_id(),
            data: self.data.clone(),
        }
    }
}

impl Mesh {
    pub fn get_unique_id(&self) -> usize {
        self.id
    }

    pub fn get_vertices(&self) -> &[Vec3] {
        &self.data.vertices
    }

    pub fn new(data: MeshData) -> Self {
        Self {
            id: gen_mesh_id(),
            data: Rc::new(data),
        }
    }
}

fn gen_mesh_id() -> usize {
    let mut rng = SmallRng::from_entropy();
    rng.gen()
}

impl DynComponent for Mesh {
    fn get_dyn_component_id(&self) -> ComponentTypeId {
        Self::COMPONENT_ID
    }
}

impl Component for Mesh {
    const COMPONENT_ID: ComponentTypeId = ID_MESH;
}
