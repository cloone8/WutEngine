use core::fmt::Display;

use glam::Vec3;
use nohash_hasher::IsEnabled;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::hash::Hash;

#[derive(Debug)]
pub struct MeshData {
    /// Unique mesh identifier
    id: MeshDataId,
    pub positions: Vec<Vec3>,
    pub indices: IndexBuffer,
}

#[derive(Debug, Clone)]
pub enum IndexBuffer {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Default for MeshData {
    fn default() -> Self {
        Self {
            id: MeshDataId::random(),
            positions: Vec::new(),
            indices: IndexBuffer::U16(Vec::new()),
        }
    }
}

impl Clone for MeshData {
    fn clone(&self) -> Self {
        Self {
            id: MeshDataId::random(),
            positions: self.positions.clone(),
            indices: self.indices.clone(),
        }
    }
}

impl MeshData {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub const fn get_id(&self) -> MeshDataId {
        self.id
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshDataId(u64);

impl Display for MeshDataId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Wrong size will mess up the format string.
        debug_assert_eq!(
            8,
            size_of::<MeshDataId>(),
            "Mesh Data ID size different from expected for 16 character hex string"
        );

        write!(f, "{:016x}", self.0)
    }
}

impl Hash for MeshDataId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl IsEnabled for MeshDataId {}

impl MeshDataId {
    pub fn random() -> Self {
        let mut rng = SmallRng::from_os_rng();
        Self(rng.random())
    }
}
