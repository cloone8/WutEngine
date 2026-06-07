//! Mesh asset

use nohash_hasher::IntMap;
use serde::Deserialize;
use serde::Serialize;
use wutengine_math::Color;
use wutengine_math::Vec2;
use wutengine_math::Vec3;

use crate::SerializedAsset;

/// The data for a single mesh
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedMesh {
    /// The vertex positions
    pub vertices: Vec<Vec3>,

    /// The topology
    pub topology: MeshTopology,

    /// The mesh index buffer. Each index should be smaller than the length of [Self::vertices]
    pub indices: MeshIndices,

    /// The UV channels. Each channel should contain exactly as much elements as [Self::vertices], or be empty
    pub uvs: IntMap<u8, Vec<Vec2>>,

    /// Color data. Should contain exactly as much elements as [Self::vertices], or be empty
    pub colors: Vec<Color>,

    /// Whether the data should be kept on the CPU after the GPU side mesh is created
    pub keep_data: bool,
}

impl SerializedAsset for SerializedMesh {}

/// Mesh indices
#[derive(Debug, Clone, derive_more::From, Serialize, Deserialize)]
pub enum MeshIndices {
    /// 16-bit indices
    U16(Vec<u16>),

    /// 32-bit indices
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
