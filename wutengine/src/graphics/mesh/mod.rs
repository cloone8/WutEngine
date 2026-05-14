//! Mesh related functionality.

mod index;
mod vertex;

use std::collections::HashMap;

use glam::{Vec2, Vec3};
pub use index::*;
use nohash_hasher::IntMap;
pub use vertex::*;

use super::shader::{GVec2, GVec3, ShaderVertexAttributeType};

/// The topology of the indices of a [Mesh]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Display)]
#[display(rename_all = "lowercase")]
pub enum MeshTopology {
    /// Triangles. 3 indices per primitive
    #[default]
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
pub struct Mesh {
    /// The vertex buffers
    pub(crate) vertex_buffers: HashMap<ShaderVertexAttributeType, VertexBuffer>,

    /// The index buffer
    pub(crate) index_buffer: IndexBuffer,
}

impl Mesh {
    pub fn new(data: MeshData) -> Option<Self> {
        let device = super::device();

        let vtx_count = data.vertices.len();

        if vtx_count == 0 {
            return None;
        }

        let vtx_pos_buffer = Vec::from_iter(data.vertices.into_iter().map(GVec3::<f32>::from));

        let pos_buffer = VertexBuffer::new(
            &vtx_pos_buffer,
            ShaderVertexAttributeType::Position,
            device,
            false,
        )
        .expect("Failed to create position buffer");

        let index_buffer = match data.indices {
            MeshDataIndices::U16(items) => {
                make_index_buffer(items, vtx_count, data.topology, device)
            }
            MeshDataIndices::U32(items) => {
                make_index_buffer(items, vtx_count, data.topology, device)
            }
        }
        .expect("Failed to create index buffer")?;

        let mut mesh = Mesh {
            vertex_buffers: HashMap::default(),
            index_buffer,
        };

        mesh.vertex_buffers
            .insert(ShaderVertexAttributeType::Position, pos_buffer);

        for (channel, data) in data.uvs {
            if data.len() != vtx_count {
                log::warn!(
                    "Discarding UV channel {channel} because it did not have the expected number of elements ({vtx_count} vertices, {} given)",
                    data.len()
                );
                continue;
            }

            let uv_vec = Vec::from_iter(data.into_iter().map(GVec2::<f32>::from));

            let uv_vtx_buf = VertexBuffer::new(
                &uv_vec,
                ShaderVertexAttributeType::Uv { channel },
                device,
                false,
            )
            .expect("Failed to create UV vertex buffer");

            mesh.vertex_buffers
                .insert(ShaderVertexAttributeType::Uv { channel }, uv_vtx_buf);
        }

        Some(mesh)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub topology: MeshTopology,
    pub indices: MeshDataIndices,
    pub uvs: IntMap<u8, Vec<Vec2>>,
}

impl MeshData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vec3>) -> &mut Self {
        self.vertices = vertices;
        self
    }
    pub fn set_indices(&mut self, indices: impl Into<MeshDataIndices>) -> &mut Self {
        self.indices = indices.into();
        self
    }

    pub fn set_topology(&mut self, topology: MeshTopology) -> &mut Self {
        self.topology = topology;
        self
    }

    pub fn set_uv_channel(&mut self, channel: u8, uvs: Vec<Vec2>) -> &mut Self {
        self.uvs.insert(channel, uvs);
        self
    }

    pub fn set_uvs(&mut self, uvs: Vec<Vec2>) -> &mut Self {
        self.set_uv_channel(0, uvs)
    }

    pub fn create(&self) -> Option<Mesh> {
        Mesh::new(self.clone())
    }
}

#[derive(Debug, Clone, derive_more::From)]
pub enum MeshDataIndices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Default for MeshDataIndices {
    fn default() -> Self {
        Self::U16(Vec::new())
    }
}

fn make_index_buffer(
    data: Vec<impl IndexDatatype>,
    num_verts: usize,
    topology: MeshTopology,
    device: &wgpu::Device,
) -> Result<Option<IndexBuffer>, NewIndexBufferErr> {
    let index_data = trim_to_multiple_of(&data, topology);

    if index_data.is_empty() {
        return Ok(None);
    }

    for index in index_data {
        let as_usize = index.as_usize();
        if as_usize >= num_verts {
            log::error!("Index {as_usize} out of range for mesh with {num_verts} vertices");
            return Ok(None);
        }
    }

    Ok(Some(IndexBuffer::new(index_data, topology, device)?))
}

fn trim_to_multiple_of<T>(data: &[T], topology: MeshTopology) -> &[T] {
    &data[..(data.len() - (data.len() % topology.indices_per_primitive()))]
}
