//! Mesh related functionality.

use std::collections::HashMap;

use nohash_hasher::IntMap;
use wutengine_assets::FromSerializedAsset;
use wutengine_assets::assets::mesh::MeshIndices;
use wutengine_assets::assets::mesh::MeshTopology;
use wutengine_assets::assets::mesh::SerializedMesh;
use wutengine_assets::assets::shader::ShaderVertexAttributeType;

use crate::shader::GVec4;

use super::shader::{GVec2, GVec3};

mod index;
pub use index::*;

mod vertex;
pub use vertex::*;

/// A raw WutEngine mesh. A collection of GPU buffers for the indices and different vertex data types.
#[derive(Debug)]
pub struct Mesh {
    /// The vertex buffers
    pub vertex_buffers: IntMap<ShaderVertexAttributeType, VertexBuffer>,

    /// The index buffer
    pub index_buffer: IndexBuffer,
}

/// Public API
impl Mesh {
    /// Returns the topology of this mesh
    pub fn topology(&self) -> MeshTopology {
        self.index_buffer.topology()
    }
}

/// Internal API
impl Mesh {
    /// Create a new native mesh from the provided serialized mesh data
    pub(crate) fn new(data: &SerializedMesh) -> Option<Self> {
        profiling::function_scope!();

        let device = super::device();

        let vtx_count = data.vertices.len();

        if vtx_count == 0 {
            return None;
        }

        let vtx_pos_buffer = data
            .vertices
            .iter()
            .copied()
            .map(GVec3::<f32>::from)
            .collect::<Vec<_>>();

        let pos_buffer = VertexBuffer::new(
            &vtx_pos_buffer,
            ShaderVertexAttributeType::Position,
            device,
            data.keep_data,
            false,
        )
        .expect("Failed to create position buffer");

        let index_buffer = match &data.indices {
            MeshIndices::U16(items) => {
                make_index_buffer(items, vtx_count, data.topology, device, data.keep_data)
            }
            MeshIndices::U32(items) => {
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

        if !data.colors.is_empty() {
            if data.colors.len() == vtx_count {
                let color_buffer = data
                    .colors
                    .iter()
                    .copied()
                    .map(|color| GVec4::<f32>::from(color.as_vec4()))
                    .collect::<Vec<_>>();

                let color_vertex_buffer = VertexBuffer::new(
                    &color_buffer,
                    ShaderVertexAttributeType::Color,
                    device,
                    data.keep_data,
                    false,
                )
                .expect("Failed to create color buffer");

                mesh.vertex_buffers
                    .insert(ShaderVertexAttributeType::Color, color_vertex_buffer);
            } else {
                log::error!(
                    "Discarding color channel because it did not have the expected number of elements ({vtx_count} vertices, {} given)",
                    data.colors.len()
                );
            }
        }

        for (&channel, uv_data) in &data.uvs {
            if uv_data.len() != vtx_count {
                log::error!(
                    "Discarding UV channel {channel} because it did not have the expected number of elements ({vtx_count} vertices, {} given)",
                    uv_data.len()
                );
                continue;
            }

            let uv_vec = uv_data
                .iter()
                .copied()
                .map(GVec2::<f32>::from)
                .collect::<Vec<_>>();

            let uv_vtx_buf = VertexBuffer::new(
                &uv_vec,
                ShaderVertexAttributeType::Uv { channel },
                device,
                data.keep_data,
                false,
            )
            .expect("Failed to create UV vertex buffer");

            mesh.vertex_buffers
                .insert(ShaderVertexAttributeType::Uv { channel }, uv_vtx_buf);
        }

        Some(mesh)
    }
}

/// Error while deserializing [`SerializedMesh`] into a [`Mesh`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display, derive_more::Error)]
pub enum MeshFromDataErr {
    /// Mesh had no vertices or no indices
    #[display("The mesh had no vertices or no indices")]
    Empty,
}

impl FromSerializedAsset for Mesh {
    type Error = MeshFromDataErr;

    type Serialized = SerializedMesh;

    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error> {
        Self::new(&serialized).ok_or(MeshFromDataErr::Empty)
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
        false,
    )?))
}

fn trim_to_multiple_of<T>(data: &[T], topology: MeshTopology) -> &[T] {
    &data[..(data.len() - (data.len() % topology.indices_per_primitive()))]
}

/// Converts a WutEngine [`MeshTopology`] to a [`wgpu::PrimitiveTopology`]
pub const fn asset_topology_to_wgpu(asset_topology: MeshTopology) -> wgpu::PrimitiveTopology {
    match asset_topology {
        MeshTopology::Triangle => wgpu::PrimitiveTopology::TriangleList,
        MeshTopology::Line => wgpu::PrimitiveTopology::LineList,
        MeshTopology::Point => wgpu::PrimitiveTopology::PointList,
    }
}
