use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub positions: Vec<Vec3>,

    pub uvs: Vec<Vec2>,

    pub indices: IndexBuffer,

    pub geometry: Geometry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexBuffer {
    U16(Vec<u16>),

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

impl IndexBuffer {
    /// Returns the current precision of this [IndexBuffer]
    pub const fn precision(&self) -> IndexPrecision {
        match self {
            IndexBuffer::U16(_) => IndexPrecision::U16,
            IndexBuffer::U32(_) => IndexPrecision::U32,
        }
    }
}

/// Defines the meaning of the indexes in a mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Geometry {
    /// Triangle mesh
    Triangles,

    /// Line mesh
    Lines,
}

/// The precision of an [IndexBuffer]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IndexPrecision {
    /// [u16] indices
    U16,

    /// [u32] indices
    U32,
}
