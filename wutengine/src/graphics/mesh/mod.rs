//! Mesh related functionality.

mod index;
mod vertex;

use std::collections::HashMap;
use std::convert::Infallible;

use glam::{Vec2, Vec3};
pub use index::*;
use nohash_hasher::IntMap;
use serde::{Deserialize, Serialize};
pub use vertex::*;

use crate::asset::{Asset, SerializedAsset};

use super::shader::{GVec2, GVec3, ShaderVertexAttributeType};

/// The topology of the indices of a [Mesh]
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, derive_more::Display, Serialize, Deserialize,
)]
#[display(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
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
    pub(crate) fn new(data: &MeshData) -> Option<Self> {
        profiling::function_scope!();

        let device = super::device();

        let vtx_count = data.vertices.len();

        if vtx_count == 0 {
            return None;
        }

        let vtx_pos_buffer = Vec::from_iter(data.vertices.iter().copied().map(GVec3::<f32>::from));

        let pos_buffer = VertexBuffer::new(
            &vtx_pos_buffer,
            ShaderVertexAttributeType::Position,
            device,
            data.keep_data,
        )
        .expect("Failed to create position buffer");

        let index_buffer = match &data.indices {
            MeshDataIndices::U16(items) => {
                make_index_buffer(items, vtx_count, data.topology, device, data.keep_data)
            }
            MeshDataIndices::U32(items) => {
                make_index_buffer(items, vtx_count, data.topology, device, data.keep_data)
            }
        }
        .expect("Failed to create index buffer")?;

        let mut mesh = Mesh {
            vertex_buffers: HashMap::default(),
            index_buffer,
        };

        mesh.vertex_buffers
            .insert(ShaderVertexAttributeType::Position, pos_buffer);

        for (&channel, uv_data) in &data.uvs {
            if uv_data.len() != vtx_count {
                log::warn!(
                    "Discarding UV channel {channel} because it did not have the expected number of elements ({vtx_count} vertices, {} given)",
                    uv_data.len()
                );
                continue;
            }

            let uv_vec = Vec::from_iter(uv_data.iter().copied().map(GVec2::<f32>::from));

            let uv_vtx_buf = VertexBuffer::new(
                &uv_vec,
                ShaderVertexAttributeType::Uv { channel },
                device,
                data.keep_data,
            )
            .expect("Failed to create UV vertex buffer");

            mesh.vertex_buffers
                .insert(ShaderVertexAttributeType::Uv { channel }, uv_vtx_buf);
        }

        Some(mesh)
    }
}

/// Error while deserializing [MeshData] into a [Mesh]
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display, derive_more::Error)]
pub enum MeshFromDataErr {
    /// Mesh had no vertices or no indices
    #[display("The mesh had no vertices or no indices")]
    Empty,
}

impl Asset for Mesh {
    type Serialized = MeshData;

    type FromSerializedErr = MeshFromDataErr;

    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized,
    {
        Self::new(serialized).ok_or(MeshFromDataErr::Empty)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub topology: MeshTopology,
    pub indices: MeshDataIndices,
    pub uvs: IntMap<u8, Vec<Vec2>>,
    pub keep_data: bool,
}

impl SerializedAsset for MeshData {
    type AssetType = Mesh;
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

    pub fn set_keep_data(&mut self, keep_data: bool) -> &mut Self {
        self.keep_data = keep_data;
        self
    }
}

#[derive(Debug, Clone, derive_more::From, Serialize, Deserialize)]
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
    data: &[impl IndexDatatype],
    num_verts: usize,
    topology: MeshTopology,
    device: &wgpu::Device,
    keep_on_cpu: bool,
) -> Result<Option<IndexBuffer>, NewIndexBufferErr> {
    let index_data = trim_to_multiple_of(data, topology);

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

    Ok(Some(IndexBuffer::new(
        index_data,
        topology,
        device,
        keep_on_cpu,
    )?))
}

fn trim_to_multiple_of<T>(data: &[T], topology: MeshTopology) -> &[T] {
    &data[..(data.len() - (data.len() % topology.indices_per_primitive()))]
}
