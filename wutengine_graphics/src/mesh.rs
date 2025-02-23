use core::fmt::Display;

use glam::Vec3;
use nohash_hasher::IsEnabled;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::hash::Hash;

/// The raw data corresponding to a mesh
#[derive(Debug)]
pub struct MeshData {
    /// Unique mesh identifier
    id: MeshDataId,

    /// The vertex positions
    pub positions: Vec<Vec3>,

    /// The vertex indices
    pub indices: IndexBuffer,
}

/// An abstraction around an index buffer
#[derive(Debug, Clone)]
pub enum IndexBuffer {
    /// 16-bit indices
    U16(Vec<u16>),

    /// 32-bit indices
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
    /// A new, empty [MeshData] struct
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the unique ID of the [MeshData]
    #[inline]
    pub const fn get_id(&self) -> MeshDataId {
        self.id
    }
}

/// A unique identifier for a set of [MeshData]
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
    /// Generate a random [MeshDataId]
    pub fn random() -> Self {
        let mut rng = SmallRng::from_os_rng();
        Self(rng.random())
    }
}
