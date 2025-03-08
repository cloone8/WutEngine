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

impl From<Vec<u16>> for IndexBuffer {
    #[inline]
    fn from(value: Vec<u16>) -> Self {
        IndexBuffer::U16(value)
    }
}

impl From<Vec<u32>> for IndexBuffer {
    #[inline]
    fn from(value: Vec<u32>) -> Self {
        IndexBuffer::U32(value)
    }
}

impl From<&[u16]> for IndexBuffer {
    #[inline]
    fn from(value: &[u16]) -> Self {
        IndexBuffer::U16(Vec::from(value))
    }
}

impl From<&[u32]> for IndexBuffer {
    #[inline]
    fn from(value: &[u32]) -> Self {
        IndexBuffer::U32(Vec::from(value))
    }
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
