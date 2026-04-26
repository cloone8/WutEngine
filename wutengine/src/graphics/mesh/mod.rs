//! Mesh related functionality.

mod index;
mod vertex;

use std::collections::HashMap;

pub use index::*;
pub use vertex::*;

use super::shader::ShaderVertexAttributeType;

/// The topology of the indices of a [Mesh]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display)]
#[display(rename_all = "lowercase")]
pub enum MeshTopology {
    /// Triangles. 3 indices per primitive
    Triangle,

    /// Lines. 2 indices per primitive
    Line,

    /// Points. 1 index per primitive
    Point,
}

impl MeshTopology {
    /// The number of indices per primitive that this topology uses
    pub const fn indices_per_primitive(self) -> usize {
        match self {
            Self::Triangle => 3,
            Self::Line => 2,
            Self::Point => 1,
        }
    }

    /// Converts the topology to a [wgpu::PrimitiveTopology]
    pub(crate) const fn to_wgpu(self) -> wgpu::PrimitiveTopology {
        match self {
            Self::Triangle => wgpu::PrimitiveTopology::TriangleList,
            Self::Line => wgpu::PrimitiveTopology::LineList,
            Self::Point => wgpu::PrimitiveTopology::PointList,
        }
    }
}

/// A raw WutEngine mesh. A collection of GPU buffers for the indices and different vertex data types.
#[derive(Debug)]
pub(crate) struct Mesh {
    /// The vertex buffers
    pub(crate) vertex_buffers: HashMap<ShaderVertexAttributeType, VertexBuffer>,

    /// The index buffer
    pub(crate) index_buffer: IndexBuffer,
}
