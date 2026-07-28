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
    /// The topology
    pub topology: MeshTopology,

    /// The vertex positions
    pub vertices: Vec<Vec3>,

    /// The mesh index buffer. Each index should be smaller than the length of [`Self::vertices`]
    pub indices: MeshIndices,

    /// The vertex normals. Should contain exactly as many elements as [`Self::vertices`], or be empty
    pub normals: Vec<Vec3>,

    /// The UV channels. Each channel should contain exactly as many elements as [`Self::vertices`], or be empty
    pub uvs: IntMap<u8, Vec<Vec2>>,

    /// Color data. Should contain exactly as many elements as [`Self::vertices`], or be empty
    pub colors: Vec<Color>,

    /// Whether the data should be kept on the CPU after the GPU side mesh is created
    pub keep_data: bool,
}

impl SerializedAsset for SerializedMesh {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("65e51770-cd08-4ba8-97db-70456af5e10b")).unwrap();

    const PREFER_BINARY_SERIALIZATION: bool = true;
}

/// Mesh indices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshIndices {
    /// 16-bit indices
    U16(Vec<u16>),

    /// 32-bit indices
    U32(Vec<u32>),
}

impl MeshIndices {
    /// Returns the number of indices
    pub const fn len(&self) -> usize {
        match self {
            Self::U16(items) => items.len(),
            Self::U32(items) => items.len(),
        }
    }

    /// Returns true if `len` is zero
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::U16(items) => items.is_empty(),
            Self::U32(items) => items.is_empty(),
        }
    }
}

impl Default for MeshIndices {
    fn default() -> Self {
        Self::U16(Vec::new())
    }
}

/// The topology of the indices of a [`SerializedMesh`]
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, derive_more::Display,
)]
#[serde(rename_all = "lowercase")]
#[display(rename_all = "lowercase")]
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
