//! OpenGL mesh and mesh buffer functionality, and mappings to/from WutEngine generic mesh types

use thiserror::Error;
use wutengine_graphics::mesh::{IndexBuffer, IndexType, MeshData};

use crate::buffer::{ArrayBuffer, ElementArrayBuffer, GlBuffer};
use crate::opengl::types::{GLenum, GLint, GLsizei, GLuint, GLushort};
use crate::opengl::{self, Gl};

/// A set of OpenGL buffers holding all the data for any given mesh
#[derive(Debug)]
pub(crate) struct GlMeshBuffers {
    /// The vertex buffer
    pub(crate) vertex: GlBuffer<ArrayBuffer>,

    /// The layout of the vertex buffer
    pub(crate) vertex_layout: MeshBufferLayout,

    /// The index buffer
    pub(crate) index: GlBuffer<ElementArrayBuffer>,

    /// The amount of elements in the index buffer
    pub(crate) num_elements: usize,

    /// The type of the elements in the index buffer
    pub(crate) element_type: IndexType,

    /// The OpenGL size of the indices
    pub(crate) index_size: GLenum,
}

/// A descriptor for the layout of a mesh vertex buffer
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub(crate) struct MeshBufferLayout {
    /// The offset of the position attribute within a vertex (in bytes)
    pub position: Option<usize>,

    /// The offset of the color attribute within a vertex (in bytes)
    pub color: Option<usize>,

    /// The location of the normal attribute within a vertex (in bytes)
    pub normal: Option<usize>,

    /// The location of the UV (texture coordinate) attribute within a vertex (in bytes)
    pub uv: Option<usize>,
}

impl MeshBufferLayout {
    /// The size (in bytes) of the vertex positional data
    pub(crate) const POS_SIZE: GLint = (size_of::<f32>() * 3) as GLint;

    /// Calculates the stride between vertices for this layout
    pub(crate) const fn calculate_stride_for_layout(&self) -> GLsizei {
        let mut stride = 0;

        if self.position.is_some() {
            stride += Self::POS_SIZE;
        }

        if self.normal.is_some() {
            todo!()
        }

        if self.color.is_some() {
            todo!()
        }

        if self.uv.is_some() {
            todo!()
        }

        stride as GLsizei
    }
}

/// Error while creating the mesh buffers
#[derive(Debug, Error)]
pub(crate) enum CreateErr {
    /// Failed to create a buffer
    #[error("Failed to create an OpenGL buffer")]
    Buf(#[from] crate::buffer::CreateErr),
}

impl GlMeshBuffers {
    /// Creates a new mesh buffer set, creating the buffers
    /// but not binding any data
    pub(crate) fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let vtxbuf = GlBuffer::new(gl)?;
        let idxbuf = GlBuffer::new(gl);

        if let Err(e) = idxbuf {
            vtxbuf.destroy(gl);
            return Err(e.into());
        }

        let idxbuf = idxbuf.unwrap();

        Ok(Self {
            vertex: vtxbuf,
            vertex_layout: MeshBufferLayout::default(),
            index: idxbuf,
            element_type: IndexType::Triangles,
            num_elements: 0,
            index_size: opengl::UNSIGNED_INT,
        })
    }

    /// Uploads the given data to this set of OpenGL mesh buffers. Discards
    /// the current data and fully replaces it with the new data.
    /// Note that this might change the mesh vertex buffer layout
    pub(crate) fn upload_data(&mut self, gl: &Gl, data: &MeshData) {
        self.vertex.bind(gl);
        self.vertex.buffer_data(gl, &data.positions);
        self.vertex.unbind(gl);

        self.vertex_layout = MeshBufferLayout {
            position: Some(0),
            ..Default::default()
        };

        self.index.bind(gl);
        match &data.indices {
            IndexBuffer::U16(items) => {
                self.index.buffer_data(gl, items);
                self.num_elements = items.len();
                self.index_size = index_size_to_gl::<u16>();
            }
            IndexBuffer::U32(items) => {
                self.index.buffer_data(gl, items);
                self.num_elements = items.len();
                self.index_size = index_size_to_gl::<u32>();
            }
        }
        self.index.unbind(gl);

        self.element_type = data.index_type;

        let expected_elements_divisor = match data.index_type {
            IndexType::Triangles => 3,
            IndexType::Lines => 2,
        };

        debug_assert_eq!(0, self.num_elements % expected_elements_divisor);
    }

    /// Destroys this mesh buffer set, freeing the GPU resources
    pub(crate) fn destroy(self, gl: &Gl) {
        self.vertex.destroy(gl);
        self.index.destroy(gl);
    }
}

const fn index_size_to_gl<T>() -> GLenum {
    const USHORT_SIZE: usize = size_of::<GLushort>();
    const UINT_SIZE: usize = size_of::<GLuint>();

    match size_of::<T>() {
        USHORT_SIZE => opengl::UNSIGNED_SHORT,
        UINT_SIZE => opengl::UNSIGNED_INT,
        _ => panic!("Unknown index size"),
    }
}

/// Convers the given general index type to an OpenGL enum
pub(crate) const fn index_type_to_gl(idxtype: IndexType) -> GLenum {
    match idxtype {
        IndexType::Triangles => opengl::TRIANGLES,
        IndexType::Lines => opengl::LINES,
    }
}
