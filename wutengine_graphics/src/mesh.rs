//! Mesh data for the WutEngine graphics APIs

use glam::Vec3;

/// The raw data corresponding to a mesh
#[derive(Debug, Clone)]
pub struct MeshData {
    /// The vertex positions
    pub positions: Vec<Vec3>,

    /// The vertex indices
    pub indices: IndexBuffer,

    /// The type of indices
    pub index_type: IndexType,
}

/// An abstraction around an index buffer
#[derive(Debug, Clone)]
pub enum IndexBuffer {
    /// 16-bit indices
    U16(Vec<u16>),

    /// 32-bit indices
    U32(Vec<u32>),
}

/// Defines the meaning of the indexes in a mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexType {
    /// Triangle mesh
    Triangles,

    /// Line mesh
    Lines,
}

impl MeshData {
    /// A new, empty [MeshData] struct
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            indices: IndexBuffer::U16(Vec::new()),
            index_type: IndexType::Triangles,
        }
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}
