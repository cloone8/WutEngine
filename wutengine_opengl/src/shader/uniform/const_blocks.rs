//! The std140 layout constant blocks used in many WutEngine shaders

use crate::gltypes::GlMat4f;

/// The viewport constants in the correct layout
/// Do not change unless these are changed in each
/// shader too
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct WutEngineViewportConstants {
    /// The view matrix
    pub(crate) view_mat: GlMat4f,

    /// The projection matrix
    pub(crate) projection_mat: GlMat4f,

    /// The view and projection matrices combined
    pub(crate) view_projection_mat: GlMat4f,
}

/// The instance constants in the correct layout
/// Do not change unless these are changed in each shader too
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct WutEngineInstanceConstants {
    /// The model matrix
    pub(crate) model_mat: GlMat4f,
}
