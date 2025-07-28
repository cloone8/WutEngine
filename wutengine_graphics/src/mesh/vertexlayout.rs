use wgpu::{VertexAttribute, VertexFormat};

use crate::format::{VTX_COLOR, VTX_NORMAL, VTX_POS, VTX_UV};
use crate::shader::ShaderVertexLayout;

/// A descriptor for the layout of a mesh vertex buffer
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct MeshVertexLayout {
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

fn set_attr(
    buf: &mut [VertexAttribute],
    shader_location: Option<u32>,
    mesh_offset: Option<u64>,
    num_attrs: &mut usize,
    format: VertexFormat,
) -> bool {
    if let Some(shader_attr_loc) = shader_location {
        let mesh_attr_offset = if let Some(mesh_attr_offset) = mesh_offset {
            mesh_attr_offset
        } else {
            return false;
        };

        buf[*num_attrs] = VertexAttribute {
            offset: mesh_attr_offset,
            shader_location: shader_attr_loc,
            format,
        };

        *num_attrs += 1;
    }

    true
}

/// Tries to create a vertex buffer layout for the combination of the given mesh and shader vertex layouts.
/// If the mesh is missing vertex attributes, will return [None] instead.
pub fn create_vertex_buffer_layout<'a>(
    buf: &'a mut [VertexAttribute],
    mesh_layout: &MeshVertexLayout,
    shader_layout: &ShaderVertexLayout,
) -> Option<wgpu::VertexBufferLayout<'a>> {
    assert!(buf.len() >= shader_layout.num_attrs(), "Buffer too small");

    let mut num_attrs = 0;

    if !set_attr(
        buf,
        shader_layout.position,
        mesh_layout.position,
        &mut num_attrs,
        VTX_POS,
    ) {
        log::error!(
            "Could not create vertex buffer layout because the mesh is missing the position attribute"
        );
        return None;
    }

    if !set_attr(
        buf,
        shader_layout.normal,
        mesh_layout.normal,
        &mut num_attrs,
        VTX_NORMAL,
    ) {
        log::error!(
            "Could not create vertex buffer layout because the mesh is missing the vertex normal attribute"
        );
        return None;
    }

    if !set_attr(
        buf,
        shader_layout.uv,
        mesh_layout.uv,
        &mut num_attrs,
        VTX_UV,
    ) {
        log::error!(
            "Could not create vertex buffer layout because the mesh is missing the UV attribute"
        );
        return None;
    }

    if !set_attr(
        buf,
        shader_layout.color,
        mesh_layout.color,
        &mut num_attrs,
        VTX_COLOR,
    ) {
        log::error!(
            "Could not create vertex buffer layout because the mesh is missing the vertex color attribute"
        );
        return None;
    }

    Some(wgpu::VertexBufferLayout {
        array_stride: mesh_layout.stride,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &buf[..num_attrs],
    })
}
