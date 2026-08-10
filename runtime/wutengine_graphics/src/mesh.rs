//! Mesh related functionality.

use core::num::NonZero;
use core::num::NonZeroU32;
use std::collections::HashMap;

use nohash_hasher::IntMap;
use wutengine_assets::FromSerializedAsset;
use wutengine_assets::assets::material::CullMode;
use wutengine_assets::assets::mesh::MeshIndices;
use wutengine_assets::assets::mesh::MeshTopology;
use wutengine_assets::assets::mesh::SerializedMesh;
use wutengine_assets::assets::shader::ShaderVertexAttributeType;

use crate::label;
use crate::shader::GVec2;
use crate::shader::GVec3;
use crate::shader::GVec4;

/// A raw WutEngine mesh. A collection of GPU buffers for the indices and different vertex data types.
#[derive(Debug)]
pub struct Mesh {
    /// The vertex buffers
    pub vertex_buffers: IntMap<ShaderVertexAttributeType, wgpu::Buffer>,

    /// The amount of vertices
    pub vertex_count: usize,

    /// The index buffer
    pub index_buffer: wgpu::Buffer,

    /// The amount of indices
    pub index_count: NonZeroU32,

    /// The type of indices
    pub index_type: IndexFormat,

    /// The mesh topology
    pub topology: MeshTopology,
}

/// Public API
impl Mesh {
    /// Returns the topology of this mesh
    pub fn topology(&self) -> MeshTopology {
        self.topology
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

        let index_buffer =
            Self::upload_index_buffer(&data.indices, data.topology, vtx_count, device)?;

        let mut mesh = Mesh {
            vertex_buffers: HashMap::default(),
            vertex_count: vtx_count,
            index_buffer,
            index_count: NonZero::new(u32::try_from(data.indices.len()).expect("Too many indices"))
                .unwrap(),
            index_type: index_format(&data.indices),
            topology: data.topology,
        };

        mesh.upload_vertex_buffer(
            label!("Vertex buffer pos"),
            ShaderVertexAttributeType::Position,
            &data.vertices,
            device,
        );

        if !data.normals.is_empty() {
            mesh.upload_vertex_buffer(
                label!("Vertex buffer normals"),
                ShaderVertexAttributeType::Normal,
                &data.normals,
                device,
            );
        }

        if !data.colors.is_empty() {
            mesh.upload_vertex_buffer(
                label!("Vertex buffer colors"),
                ShaderVertexAttributeType::Color,
                &data.colors,
                device,
            );
        }

        for (&channel, uv_data) in &data.uvs {
            if uv_data.is_empty() {
                continue;
            }

            mesh.upload_vertex_buffer(
                label!("Vertex buffer UV {}", channel),
                ShaderVertexAttributeType::Uv { channel },
                uv_data.as_slice(),
                device,
            );
        }

        Some(mesh)
    }

    fn upload_index_buffer(
        indices: &MeshIndices,
        topology: MeshTopology,
        vtx_count: usize,
        device: &wgpu::Device,
    ) -> Option<wgpu::Buffer> {
        profiling::function_scope!();

        if !indices
            .len()
            .is_multiple_of(topology.indices_per_primitive())
        {
            panic!("Index count is not a multiple of the primitive type index count");
        }

        if indices.is_empty() {
            return None;
        }

        let indices_bytes = match indices {
            MeshIndices::U16(items) => {
                for index in items.iter().copied() {
                    if index >= (vtx_count as u16) {
                        log::error!(
                            "Index {index} out of range for mesh with {vtx_count} vertices"
                        );
                        return None;
                    }
                }

                bytemuck::must_cast_slice::<u16, u8>(items.as_slice())
            }
            MeshIndices::U32(items) => {
                for index in items.iter().copied() {
                    if index >= (vtx_count as u32) {
                        log::error!(
                            "Index {index} out of range for mesh with {vtx_count} vertices"
                        );
                        return None;
                    }
                }

                bytemuck::must_cast_slice::<u32, u8>(items.as_slice())
            }
        };

        let index_buffer_size_aligned =
            (indices_bytes.len() as u64).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT);

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: label!("Index buffer"),
            size: index_buffer_size_aligned,
            usage: wgpu::BufferUsages::INDEX,
            mapped_at_creation: true,
        });

        index_buffer
            .get_mapped_range_mut(..)
            .expect("Failed to get mapped index buffer")
            .slice(..indices_bytes.len())
            .copy_from_slice(indices_bytes);

        index_buffer.unmap();

        Some(index_buffer)
    }

    fn upload_vertex_buffer<T>(
        &mut self,
        label: Option<&str>,
        attr: ShaderVertexAttributeType,
        data: &[T],
        device: &wgpu::Device,
    ) where
        T: bytemuck::NoUninit,
    {
        profiling::function_scope!();

        const {
            assert!(
                size_of::<T>().is_multiple_of(wgpu::VERTEX_ALIGNMENT as usize),
                "Incorrect alignment"
            );
        }

        assert_eq!(attr_bytes(attr), size_of::<T>(), "Attribute size mismatch");

        if data.is_empty() {
            return;
        }

        let vertex_buffer_bytes = bytemuck::must_cast_slice::<T, u8>(data);

        let vertex_buffer_size_aligned =
            (data.len() as u64).next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT);

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size: vertex_buffer_size_aligned,
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: true,
        });

        vertex_buffer
            .get_mapped_range_mut(..)
            .expect("Failed to get mapped vertex buffer")
            .slice(..vertex_buffer_bytes.len())
            .copy_from_slice(vertex_buffer_bytes);

        vertex_buffer.unmap();

        self.vertex_buffers.insert(attr, vertex_buffer);
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

/// Converts a WutEngine [`MeshTopology`] to a [`wgpu::PrimitiveTopology`]
pub const fn asset_topology_to_wgpu(asset_topology: MeshTopology) -> wgpu::PrimitiveTopology {
    match asset_topology {
        MeshTopology::Triangle => wgpu::PrimitiveTopology::TriangleList,
        MeshTopology::Line => wgpu::PrimitiveTopology::LineList,
        MeshTopology::Point => wgpu::PrimitiveTopology::PointList,
    }
}

/// Converts a WutEngine [`CullMode`] to a [`wgpu::Face`], or [`None`]
pub const fn asset_cull_mode_to_wgpu(asset_cull_mode: CullMode) -> Option<wgpu::Face> {
    match asset_cull_mode {
        CullMode::Front => Some(wgpu::Face::Front),
        CullMode::Back => Some(wgpu::Face::Back),
        CullMode::None => None,
    }
}

/// The format of the index buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexFormat {
    /// 16-bit indices
    U16,

    /// 32-bit indices
    U32,
}

impl IndexFormat {
    /// Converts the index format to its [`wgpu::IndexFormat`] equivalent
    pub const fn to_wgpu(self) -> wgpu::IndexFormat {
        match self {
            Self::U16 => wgpu::IndexFormat::Uint16,
            Self::U32 => wgpu::IndexFormat::Uint32,
        }
    }

    /// The stride in bytes per element of this format
    pub const fn stride(self) -> usize {
        match self {
            Self::U16 => size_of::<u16>(),
            Self::U32 => size_of::<u32>(),
        }
    }
}

const fn index_format(indices: &MeshIndices) -> IndexFormat {
    match indices {
        MeshIndices::U16(_) => IndexFormat::U16,
        MeshIndices::U32(_) => IndexFormat::U32,
    }
}

/// Returns the amount of bytes used by a given shader vertex attribute
pub const fn attr_bytes(attr: ShaderVertexAttributeType) -> usize {
    match attr {
        ShaderVertexAttributeType::Position => size_of::<GVec3<f32>>(),
        ShaderVertexAttributeType::Normal => size_of::<GVec3<f32>>(),
        ShaderVertexAttributeType::Uv { .. } => size_of::<GVec2<f32>>(),
        ShaderVertexAttributeType::Color => size_of::<GVec4<f32>>(),
    }
}
