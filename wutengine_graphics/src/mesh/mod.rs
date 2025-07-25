use core::num::NonZeroU64;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use wgpu::wgt::BufferDescriptor;
use wgpu::{Buffer, BufferUsages};
use wutengine_asset::Asset;

use crate::color::Color;
use crate::resource::GpuResource;
use crate::{GRAPHICS_MANAGER, format};

mod vertexlayout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    colors: Vec<Color>,
    indices: IndexBuffer,
    geometry: Geometry,

    #[serde(skip)]
    vertex_buffer: GpuResource<Buffer>,

    #[serde(skip)]
    index_buffer: GpuResource<Buffer>,
}

impl Asset for Mesh {
    const FORCE_BINARY: bool = true;
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            colors: Vec::new(),
            indices: IndexBuffer::U16(Vec::new()),
            geometry: Geometry::Triangles,
            vertex_buffer: GpuResource::new(),
            index_buffer: GpuResource::new(),
        }
    }
}

impl Mesh {
    pub fn get_layout(&self) -> MeshVertexLayout {
        let mut cur_offset = 0;
        let mut layout = MeshVertexLayout::EMPTY;

        if !self.positions.is_empty() {
            layout.position = Some(cur_offset);
            cur_offset += format::VTX_POS.size();
        }

        if !self.normals.is_empty() {
            layout.normal = Some(cur_offset);
            cur_offset += format::VTX_NORMAL.size();
        }

        if !self.uvs.is_empty() {
            layout.uv = Some(cur_offset);
            cur_offset += format::VTX_UV.size();
        }

        if !self.colors.is_empty() {
            layout.color = Some(cur_offset);
            cur_offset += format::VTX_COLOR.size();
        }

        layout.stride = cur_offset.next_multiple_of(wgpu::VERTEX_STRIDE_ALIGNMENT);

        layout
    }

    fn check_data_sizes(&self) -> bool {
        let mut expected_size: Option<usize> = None;

        if !self.positions.is_empty() {
            expected_size = Some(self.positions.len());
        }

        if !self.normals.is_empty() {
            match expected_size {
                Some(expected) => {
                    if self.normals.len() != expected {
                        log::error!(
                            "Amount of vertex normals ({}) does not match the amount of vertices ({}). Not rendering",
                            self.normals.len(),
                            expected
                        );
                        return false;
                    }
                }
                None => expected_size = Some(self.normals.len()),
            }
        }

        if !self.uvs.is_empty() {
            match expected_size {
                Some(expected) => {
                    if self.uvs.len() != expected {
                        log::error!(
                            "Amount of UVs ({}) does not match the amount of vertices ({}). Not rendering",
                            self.uvs.len(),
                            expected
                        );
                        return false;
                    }
                }
                None => expected_size = Some(self.uvs.len()),
            }
        }

        if !self.colors.is_empty() {
            if let Some(expected) = expected_size {
                if self.colors.len() != expected {
                    log::error!(
                        "Amount of colors ({}) does not match the amount of vertices ({}). Not rendering",
                        self.colors.len(),
                        expected
                    );
                    return false;
                }
            }
        }

        true
    }

    #[profiling::function]
    fn update_vertex_buffer(&mut self) {
        if !self.check_data_sizes() {
            // Data is invalid. Do not update
            return;
        }

        let layout = self.get_layout();
        let num_vertices = self.num_vertices() as u64;

        if num_vertices == 0 {
            // No vertices, so nothing to do
            self.vertex_buffer.clear();
            return;
        }

        let expected_buffer_size = NonZeroU64::new(num_vertices * layout.stride).unwrap();

        let needs_new_buffer = match self.vertex_buffer.get() {
            Some(vtx_buf) => vtx_buf.size() < expected_buffer_size.get(),
            None => true,
        };

        if needs_new_buffer {
            profiling::scope!("Create new buffer");
            log::debug!("Creating new vertex buffer");

            self.vertex_buffer
                .set(GRAPHICS_MANAGER.device.create_buffer(&BufferDescriptor {
                    label: Some("Mesh Vertex Buffer"),
                    size: expected_buffer_size.get(),
                    usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }

        {
            profiling::scope!("Write into buffer");

            let mut bufferview = GRAPHICS_MANAGER
                .queue
                .write_buffer_with(self.vertex_buffer.get().unwrap(), 0, expected_buffer_size)
                .expect("Failed to obtain writable buffer view");

            // Copy each vertex into the view
            for vtx in 0..num_vertices {
                let vtx_offset = vtx * layout.stride;

                // Positions
                if let Some(pos_offset) = layout.position {
                    let pos_offset = (vtx_offset + pos_offset) as usize;
                    let pos_end = pos_offset + format::VTX_POS.size() as usize;

                    bufferview[pos_offset..pos_end]
                        .copy_from_slice(bytemuck::bytes_of(&self.positions[vtx as usize]));
                }

                // Normals
                if let Some(norm_offset) = layout.normal {
                    let norm_offset = (vtx_offset + norm_offset) as usize;
                    let norm_end = norm_offset + format::VTX_NORMAL.size() as usize;

                    bufferview[norm_offset..norm_end]
                        .copy_from_slice(bytemuck::bytes_of(&self.normals[vtx as usize]));
                }

                // UVs
                if let Some(uv_offset) = layout.uv {
                    let uv_offset = (vtx_offset + uv_offset) as usize;
                    let uv_end = uv_offset + format::VTX_UV.size() as usize;

                    bufferview[uv_offset..uv_end]
                        .copy_from_slice(bytemuck::bytes_of(&self.uvs[vtx as usize]));
                }

                // Colors
                if let Some(color_offset) = layout.color {
                    let color_offset = (vtx_offset + color_offset) as usize;
                    let color_end = color_offset + format::VTX_COLOR.size() as usize;

                    bufferview[color_offset..color_end]
                        .copy_from_slice(bytemuck::bytes_of(&self.colors[vtx as usize]));
                }
            }
        }
    }

    #[profiling::function]
    fn update_index_buffer(&mut self) {
        if self.indices.is_empty() {
            self.index_buffer.clear();
        }

        let expected_buffer_size = NonZeroU64::new(
            (self.indices.len() * self.indices.precision().bytes_per_index()) as u64,
        )
        .unwrap();

        let needs_new_buffer = match self.index_buffer.get() {
            Some(idx_buf) => idx_buf.size() < expected_buffer_size.get(),
            None => true,
        };

        if needs_new_buffer {
            profiling::scope!("Create new buffer");
            log::debug!("Creating new index buffer");

            self.index_buffer
                .set(GRAPHICS_MANAGER.device.create_buffer(&BufferDescriptor {
                    label: Some("Mesh Index Buffer"),
                    size: expected_buffer_size.get(),
                    usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }

        {
            profiling::scope!("Write into buffer");

            let mut bufferview = GRAPHICS_MANAGER
                .queue
                .write_buffer_with(self.index_buffer.get().unwrap(), 0, expected_buffer_size)
                .expect("Failed to obtain writable buffer view");

            bufferview.copy_from_slice(self.indices.as_byte_slice());
        }
    }
}

/// Public API for [Mesh]
impl Mesh {
    /// Returns a new empty [Mesh] with triangle geometry
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_vertices(&self) -> usize {
        if !self.positions.is_empty() {
            self.positions.len()
        } else if !self.normals.is_empty() {
            self.normals.len()
        } else if !self.uvs.is_empty() {
            self.uvs.len()
        } else {
            0
        }
    }

    pub fn num_indices(&self) -> u32 {
        self.indices.len() as u32
    }

    /// Returns a copy of the vertex positions of this [Mesh].
    /// For a non-allocating version, see [Self::read_vertex_positions]
    pub fn get_vertex_positions(&self) -> Vec<Vec3> {
        self.positions.clone()
    }

    /// Returns the vertex positions by appending them to the current end of `buf`
    pub fn read_vertex_positions(&self, buf: &mut Vec<Vec3>) {
        buf.extend_from_slice(self.positions.as_slice());
    }

    /// Sets the vertex positions
    pub fn set_vertex_positions(&mut self, positions: Vec<Vec3>) {
        self.positions = positions;
        self.update_vertex_buffer();
    }

    /// Returns a copy of the vertex normals of this [Mesh].
    /// For a non-allocating version, see [Self::read_vertex_normals]
    pub fn get_vertex_normals(&self) -> Vec<Vec3> {
        self.normals.clone()
    }

    /// Returns the vertex normals by appending them to the current end of `buf`
    pub fn read_vertex_normals(&self, buf: &mut Vec<Vec3>) {
        buf.extend_from_slice(self.normals.as_slice());
    }

    /// Sets the vertex normals
    pub fn set_vertex_normals(&mut self, normals: Vec<Vec3>) {
        self.normals = normals;
        self.update_vertex_buffer();
    }

    /// Returns a copy of the vertex texture coordinates of this [Mesh].
    /// For a non-allocating version, see [Self::read_uvs]
    pub fn get_uvs(&self) -> Vec<Vec2> {
        self.uvs.clone()
    }

    /// Returns the vertex texture coordinates by appending them to the current end of `buf`
    pub fn read_uvs(&self, buf: &mut Vec<Vec2>) {
        buf.extend_from_slice(self.uvs.as_slice());
    }

    /// Sets the vertex positions
    pub fn set_uvs(&mut self, uvs: Vec<Vec2>) {
        self.uvs = uvs;
        self.update_vertex_buffer();
    }

    /// Returns a copy of the vertex colors of this [Mesh].
    /// For a non-allocating version, see [Self::read_colors]
    pub fn get_colors(&self) -> Vec<Color> {
        self.colors.clone()
    }

    /// Returns the vertex colors by appending them to the current end of `buf`
    pub fn read_colors(&self, buf: &mut Vec<Color>) {
        buf.extend_from_slice(self.colors.as_slice());
    }

    /// Sets the vertex colors
    pub fn set_colors(&mut self, colors: Vec<Color>) {
        self.colors = colors;
        self.update_vertex_buffer();
    }

    /// Returns a copy of the indices of this [Mesh].
    /// For a non-allocating version, see [Self::read_indices]
    pub fn get_indices(&self) -> IndexBuffer {
        self.indices.clone()
    }

    /// Returns the indices appending them to the current end of `buf`.
    /// If `buf` does not contain the same type of indices as this mesh,
    /// the indices of the mesh are cast to the type of indices in `buf`,
    /// possibly leading to overflow if casting from [u32] to [u16]
    pub fn read_indices(&self, buf: &mut IndexBuffer) {
        let idxbuf = &self.indices;

        match (buf, idxbuf) {
            // Fast conversionless cases
            (IndexBuffer::U16(output), IndexBuffer::U16(input)) => {
                output.extend_from_slice(input);
            }
            (IndexBuffer::U32(output), IndexBuffer::U32(input)) => {
                output.extend_from_slice(input);
            }

            // Cases with conversion
            (IndexBuffer::U16(output), IndexBuffer::U32(input)) => {
                output.reserve(input.len());
                output.extend(input.iter().map(|idx| *idx as u16));
            }
            (IndexBuffer::U32(output), IndexBuffer::U16(input)) => {
                output.reserve(input.len());
                output.extend(input.iter().map(|idx| *idx as u32));
            }
        }
    }

    /// Sets the index buffer
    pub fn set_indices(&mut self, indices: impl Into<IndexBuffer>) {
        self.indices = indices.into();
        self.update_index_buffer();
    }

    /// Returns the precision of the mesh [IndexBuffer]
    pub fn get_index_precision(&self) -> IndexPrecision {
        self.indices.precision()
    }

    /// Returns the type of index geometry of this mesh
    pub fn get_geometry(&self) -> Geometry {
        self.geometry
    }

    /// Sets the type of index geometry of this mesh
    pub fn set_index_type(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    pub fn get_vertex_buffer(&self) -> Option<&wgpu::Buffer> {
        self.vertex_buffer.get()
    }

    pub fn get_index_buffer(&self) -> Option<&wgpu::Buffer> {
        self.index_buffer.get()
    }
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
    #[inline(always)]
    pub const fn precision(&self) -> IndexPrecision {
        match self {
            IndexBuffer::U16(_) => IndexPrecision::U16,
            IndexBuffer::U32(_) => IndexPrecision::U32,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            IndexBuffer::U16(items) => items.len(),
            IndexBuffer::U32(items) => items.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        match self {
            IndexBuffer::U16(items) => items.is_empty(),
            IndexBuffer::U32(items) => items.is_empty(),
        }
    }

    #[inline]
    pub(crate) fn as_byte_slice(&self) -> &[u8] {
        match self {
            IndexBuffer::U16(items) => bytemuck::cast_slice(items),
            IndexBuffer::U32(items) => bytemuck::cast_slice(items),
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

impl IndexPrecision {
    pub const fn bytes_per_index(self) -> usize {
        match self {
            IndexPrecision::U16 => size_of::<u16>(),
            IndexPrecision::U32 => size_of::<u32>(),
        }
    }
}

impl From<IndexPrecision> for wgpu::IndexFormat {
    #[inline]
    fn from(value: IndexPrecision) -> Self {
        match value {
            IndexPrecision::U16 => wgpu::IndexFormat::Uint16,
            IndexPrecision::U32 => wgpu::IndexFormat::Uint32,
        }
    }
}

pub use vertexlayout::MeshVertexLayout;
pub use vertexlayout::create_vertex_buffer_layout;
