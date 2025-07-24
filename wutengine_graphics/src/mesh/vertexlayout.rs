use wgpu::VertexAttribute;

use crate::format::{VTX_COLOR, VTX_NORMAL, VTX_POS, VTX_UV};
use crate::mesh;
use crate::shader::ShaderVertexLayout;

/// A descriptor for the layout of a mesh vertex buffer
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub(crate) struct MeshVertexLayout {
    /// The total stride between consecutive vertices
    pub stride: u64,

    /// The offset of the position attribute within a vertex (in bytes)
    pub position: Option<u64>,

    /// The location of the normal attribute within a vertex (in bytes)
    pub normal: Option<u64>,

    /// The location of the UV (texture coordinate) attribute within a vertex (in bytes)
    pub uv: Option<u64>,

    /// The offset of the color attribute within a vertex (in bytes)
    pub color: Option<u64>,
}

impl MeshVertexLayout {
    pub(crate) const EMPTY: Self = Self {
        stride: 0,
        position: None,
        color: None,
        normal: None,
        uv: None,
    };

    pub(crate) const fn num_attrs(&self) -> usize {
        self.position.is_some() as usize
            + self.normal.is_some() as usize
            + self.uv.is_some() as usize
            + self.color.is_some() as usize
    }
}

pub(crate) fn create_vertex_buffer_layout<'a>(
    buf: &'a mut [VertexAttribute],
    mesh_layout: &MeshVertexLayout,
    shader_layout: &ShaderVertexLayout,
) -> wgpu::VertexBufferLayout<'a> {
    assert!(buf.len() >= shader_layout.num_attrs(), "Buffer too small");

    let mut num_attrs = 0;

    if let (Some(mesh_attr_offset), Some(shader_attr_loc)) =
        (mesh_layout.position, shader_layout.position)
    {
        buf[num_attrs] = VertexAttribute {
            offset: mesh_attr_offset,
            shader_location: shader_attr_loc,
            format: VTX_POS,
        };

        num_attrs += 1;
    }

    if let (Some(mesh_attr_offset), Some(shader_attr_loc)) =
        (mesh_layout.normal, shader_layout.normal)
    {
        buf[num_attrs] = VertexAttribute {
            offset: mesh_attr_offset,
            shader_location: shader_attr_loc,
            format: VTX_NORMAL,
        };

        num_attrs += 1;
    }

    if let (Some(mesh_attr_offset), Some(shader_attr_loc)) = (mesh_layout.uv, shader_layout.uv) {
        buf[num_attrs] = VertexAttribute {
            offset: mesh_attr_offset,
            shader_location: shader_attr_loc,
            format: VTX_UV,
        };

        num_attrs += 1;
    }

    if let (Some(mesh_attr_offset), Some(shader_attr_loc)) =
        (mesh_layout.color, shader_layout.color)
    {
        buf[num_attrs] = VertexAttribute {
            offset: mesh_attr_offset,
            shader_location: shader_attr_loc,
            format: VTX_COLOR,
        };

        num_attrs += 1;
    }

    wgpu::VertexBufferLayout {
        array_stride: mesh_layout.stride,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &buf[..num_attrs],
    }
}
