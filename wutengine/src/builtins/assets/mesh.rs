use std::sync::{Arc, RwLock};

use glam::{Vec2, Vec3};
use wutengine_graphics::mesh::{IndexBuffer, IndexPrecision, IndexType, MeshData};
use wutengine_graphics::renderer::{RendererMeshId, WutEngineRenderer};

use crate::asset::Asset;

/// A renderable mesh. Works together with a [super::Material] asset
/// to enable an entity to be rendered.
#[derive(Debug, Clone)]
pub struct Mesh(pub(crate) Arc<RwLock<RawMesh>>);

impl Mesh {
    /// Returns a copy of the vertex positions of this [Mesh].
    /// For a non-allocating version, see [Self::read_vertex_positions]
    pub fn get_vertex_positions(&self) -> Vec<Vec3> {
        self.0.read().unwrap().data.positions.clone()
    }

    /// Returns the vertex positions by appending them to the current end of `buf`
    pub fn read_vertex_positions(&self, buf: &mut Vec<Vec3>) {
        let pos = &self.0.read().unwrap().data.positions;

        buf.extend_from_slice(pos);
    }

    /// Sets the vertex positions
    pub fn set_vertex_positions(&mut self, positions: Vec<Vec3>) {
        let raw = self.get_raw_mut_cloned();
        raw.data.positions = positions;
        raw.dirty = true;
    }

    /// Returns a copy of the vertex texture coordinates of this [Mesh].
    /// For a non-allocating version, see [Self::read_uvs]
    pub fn get_uvs(&self) -> Vec<Vec2> {
        self.0.read().unwrap().data.uvs.clone()
    }

    /// Returns the vertex texture coordinates by appending them to the current end of `buf`
    pub fn read_uvs(&self, buf: &mut Vec<Vec2>) {
        let uvs = &self.0.read().unwrap().data.uvs;

        buf.extend_from_slice(uvs);
    }

    /// Sets the vertex positions
    pub fn set_uvs(&mut self, uvs: Vec<Vec2>) {
        let raw = self.get_raw_mut_cloned();
        raw.data.uvs = uvs;
        raw.dirty = true;
    }

    /// Returns a copy of the indices of this [Mesh].
    /// For a non-allocating version, see [Self::read_indices]
    pub fn get_indices(&self) -> IndexBuffer {
        self.0.read().unwrap().data.indices.clone()
    }

    /// Returns the indices appending them to the current end of `buf`.
    /// If `buf` does not contain the same type of indices as this mesh,
    /// the indices of the mesh are cast to the type of indices in `buf`,
    /// possibly leading to overflow if casting from [u32] to [u16]
    pub fn read_indices(&self, buf: &mut IndexBuffer) {
        let idxbuf = &self.0.read().unwrap().data.indices;

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
        let raw = self.get_raw_mut_cloned();
        raw.data.indices = indices.into();
        raw.dirty = true;
    }

    /// Returns the precision of the mesh [IndexBuffer]
    pub fn get_index_precision(&self) -> IndexPrecision {
        self.0.read().unwrap().data.indices.precision()
    }

    /// Returns the type of indices of this mesh
    pub fn get_index_type(&self) -> IndexType {
        self.0.read().unwrap().data.index_type
    }

    /// Sets the type of indices of this mesh
    pub fn set_index_type(&mut self, index_type: IndexType) {
        let raw = self.get_raw_mut_cloned();
        raw.data.index_type = index_type;
        raw.dirty = true;
    }

    /// Creates a new [Mesh]
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(RawMesh {
            renderer_id: RendererMeshId::new(),
            dirty: true,
            data: MeshData::default(),
        })))
    }
}

/// Private utilities
impl Mesh {
    fn get_raw_mut_cloned(&mut self) -> &mut RawMesh {
        let is_unique = Arc::get_mut(&mut self.0).is_some();

        if !is_unique {
            let new_arc = {
                let cloned = self.0.read().unwrap().clone();

                Arc::new(RwLock::new(cloned))
            };

            self.0 = new_arc;
        }

        Arc::get_mut(&mut self.0)
            .expect("Should be unique")
            .get_mut()
            .unwrap()
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Asset for Mesh {}

/// The raw internal mesh data for a [Mesh] asset
#[derive(Debug)]
pub(crate) struct RawMesh {
    /// The renderer ID for this mesh
    pub(crate) renderer_id: RendererMeshId,

    dirty: bool,

    /// The actual mesh data.
    /// Allows multiple meshes to use the same data transparently
    pub(crate) data: MeshData,
}

impl Clone for RawMesh {
    fn clone(&self) -> Self {
        Self {
            renderer_id: RendererMeshId::new(),
            dirty: true,
            data: self.data.clone(),
        }
    }
}

impl RawMesh {
    /// Flushes the changes on this mesh to the given renderer, if needed
    pub(crate) fn flush(&mut self, renderer: &mut impl WutEngineRenderer) {
        if !self.dirty {
            return;
        }

        renderer.update_mesh(self.renderer_id, &self.data);

        self.dirty = false;
    }

    /// Flushes the changes on this mesh to the given renderer if needed, and
    /// returns its ID
    pub(crate) fn flush_and_get_id(
        this: &Arc<RwLock<Self>>,
        renderer: &mut impl WutEngineRenderer,
    ) -> RendererMeshId {
        let mut locked = this.write().unwrap();

        locked.flush(renderer);

        locked.renderer_id
    }
}
