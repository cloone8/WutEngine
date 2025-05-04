use crate::gltypes::GlMat4f;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct WutEngineViewportConstants {
    pub(crate) view_mat: GlMat4f,
    pub(crate) projection_mat: GlMat4f,
    pub(crate) view_projection_mat: GlMat4f,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WutEngineInstanceConstants {
    pub(crate) model_mat: GlMat4f,
}
