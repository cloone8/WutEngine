//! Internal WutEngine graphics functionality

use std::sync::{Arc, RwLock};

use glam::Mat4;

use crate::builtins::assets::{RawMaterial, RawMesh};

pub(crate) mod objects;
pub(crate) mod viewports;

/// A render command to be sent to the graphics backend
#[derive(Debug, Clone)]
pub(crate) struct RenderCommand {
    /// The mesh to render
    pub(crate) mesh: Arc<RwLock<RawMesh>>,

    /// The material to render with
    pub(crate) material: Arc<RwLock<RawMaterial>>,

    /// The mesh object-to-world matrix
    pub(crate) object_to_world: Mat4,
}
