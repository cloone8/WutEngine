use glam::Vec3;
use rand::{rngs::SmallRng, Rng, SeedableRng};

#[derive(Debug)]
pub struct MeshData {
    /// Unique mesh identifier
    id: usize,
    pub vertices: Vec<Vec3>,
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            id: MeshData::random_id(),
            vertices: Vec::new(),
        }
    }
}

impl Clone for MeshData {
    fn clone(&self) -> Self {
        Self {
            id: MeshData::random_id(),
            vertices: self.vertices.clone(),
        }
    }
}

impl MeshData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    fn random_id() -> usize {
        let mut rng = SmallRng::from_entropy();
        rng.gen()
    }
}
