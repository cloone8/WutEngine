use wutengine_graphics::mesh::{IndexBuffer, MeshData};

use crate::buffer::{ArrayBuffer, CreateErr, ElementArrayBuffer, GlBuffer};
use crate::opengl::Gl;

use super::LayoutDescriptor;

#[derive(Debug)]
pub struct GlMeshBuffers {
    pub layout: LayoutDescriptor,
    pub vertex: GlBuffer<ArrayBuffer>,
    pub index: GlBuffer<ElementArrayBuffer>,
}

impl GlMeshBuffers {
    pub fn new(gl: &Gl, mesh: &MeshData) -> Result<Self, CreateErr> {
        let mut buf = Self {
            layout: LayoutDescriptor { position: 0 },
            vertex: GlBuffer::new(gl)?,
            index: GlBuffer::new(gl)?,
        };

        let mut vertex_buf: Vec<f32> = Vec::with_capacity(mesh.positions.len() * 3);

        for position in mesh.positions.iter().copied() {
            vertex_buf.push(position.x);
            vertex_buf.push(position.y);
            vertex_buf.push(position.z);
        }

        buf.vertex.bind(gl);
        buf.vertex.buffer_data(gl, &vertex_buf);
        buf.vertex.unbind(gl);

        let mut index_buf: Vec<u32> = Vec::new();

        match &mesh.indices {
            IndexBuffer::U16(vec) => {
                for index in vec.iter().copied() {
                    index_buf.push(index as u32);
                }
            }
            IndexBuffer::U32(vec) => {
                for index in vec.iter().copied() {
                    index_buf.push(index);
                }
            }
        }

        buf.index.bind(gl);
        buf.index.buffer_data(gl, &index_buf);
        buf.index.unbind(gl);

        Ok(buf)
    }
}
