//! Mesh asset

use glam::Vec2;
use glam::Vec3;
use nohash_hasher::IntMap;
use serde::Deserialize;
use serde::Serialize;

use crate::SerializedAsset;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedMesh {
    pub vertices: Vec<Vec3>,
    pub topology: MeshTopology,
    pub indices: MeshIndices,
    pub uvs: IntMap<u8, Vec<Vec2>>,
    pub keep_data: bool,
}

impl SerializedAsset for SerializedMesh {}

impl SerializedMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_vertices(&mut self, vertices: Vec<Vec3>) -> &mut Self {
        self.vertices = vertices;
        self
    }
    pub fn set_indices(&mut self, indices: impl Into<MeshIndices>) -> &mut Self {
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
pub enum MeshIndices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Default for MeshIndices {
    fn default() -> Self {
        Self::U16(Vec::new())
    }
}

/// The topology of the indices of a [SerializedMesh]
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
}
