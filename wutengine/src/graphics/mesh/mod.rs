mod index;
mod vertex;

use std::collections::HashMap;

pub use index::*;
pub use vertex::*;

use super::shader::ShaderVertexAttributeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display)]
#[display(rename_all = "lowercase")]
pub(crate) enum MeshTopology {
    Triangle,
    Line,
    Point,
}

impl MeshTopology {
    pub const fn indices_per_primitive(self) -> usize {
        match self {
            Self::Triangle => 3,
            Self::Line => 2,
            Self::Point => 1,
        }
    }

    pub const fn to_wgpu(self) -> wgpu::PrimitiveTopology {
        match self {
            Self::Triangle => wgpu::PrimitiveTopology::TriangleList,
            Self::Line => wgpu::PrimitiveTopology::LineList,
            Self::Point => wgpu::PrimitiveTopology::PointList,
        }
    }
}

pub(crate) struct Mesh {
    pub(crate) vertex_buffers: HashMap<ShaderVertexAttributeType, VertexBuffer>,
    pub(crate) index_buffer: IndexBuffer,
}
