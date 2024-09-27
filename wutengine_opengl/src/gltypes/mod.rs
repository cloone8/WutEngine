use core::ffi::c_void;

use glam::Vec3;
use wutengine_graphics::mesh::{IndexBuffer, MeshData};

use crate::buffer::{ArrayBuffer, CreateErr, ElementArrayBuffer, GlBuffer};
use crate::opengl::types::{GLenum, GLint, GLsizei};
use crate::opengl::Gl;
use crate::shader::attribute::ShaderAttribute;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlPosition {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3> for GlPosition {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Debug)]
pub struct GlMeshBuffers {
    pub layout: LayoutDescriptor,
    pub vertex: GlBuffer<ArrayBuffer>,
    pub index: GlBuffer<ElementArrayBuffer>,
}

#[derive(Debug)]
pub struct LayoutDescriptor {
    /// The layout index of the position data, if any
    pub position: usize,
}

#[derive(Debug)]
pub struct AttributeLayout {
    /// In number of elements
    pub size: GLint,
    pub gltype: GLenum,
    pub stride: GLsizei,
    pub offset: *const c_void,
}

impl LayoutDescriptor {
    pub fn get_present_attributes(&self) -> Vec<ShaderAttribute> {
        ShaderAttribute::ALL
            .into_iter()
            .filter(|attr| self.get_attribute_index(*attr).is_some())
            .collect()
    }

    pub fn total_size(&self) -> GLsizei {
        ShaderAttribute::Position.size_bytes()
    }

    pub fn get_attribute_index(&self, attribute: ShaderAttribute) -> Option<usize> {
        match attribute {
            ShaderAttribute::Position => Some(self.position),
        }
    }

    pub fn get_attributes_before(&self, attribute: ShaderAttribute) -> Vec<ShaderAttribute> {
        let attr_index = self.get_attribute_index(attribute).unwrap();

        self.get_present_attributes()
            .into_iter()
            .filter(|attr| *attr != attribute)
            .filter(|attr| {
                let other_index = self.get_attribute_index(*attr).unwrap();

                other_index < attr_index
            })
            .collect()
    }

    pub fn get_for_attribute(&self, attribute: ShaderAttribute) -> Option<AttributeLayout> {
        // Check if it exists at all
        let exists = self.get_attribute_index(attribute).is_some();

        if !exists {
            return None;
        }

        let size = attribute.num_components();
        let gltype = attribute.component_type();

        // To calculate stride, we need the total size of all attributes present
        // in this layout
        let stride = self.total_size();

        // To calculate the offset, we need to get all data parameters _before_ this one in the layout
        let offset: GLsizei = self
            .get_attributes_before(attribute)
            .into_iter()
            .map(|a| a.size_bytes())
            .sum();

        Some(AttributeLayout {
            size,
            gltype,
            stride,
            offset: offset as *const c_void,
        })
    }
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
