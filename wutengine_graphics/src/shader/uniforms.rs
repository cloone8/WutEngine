use core::ffi::CStr;

/// Uniforms shared between most/all shaders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedShaderUniform {
    /// The model matrix
    ModelMat,

    /// The view matrix
    ViewMat,

    /// The projection matrix
    ProjectionMat,
}

impl SharedShaderUniform {
    /// Returns the shared uniform as a [str]
    pub const fn as_str(self) -> &'static str {
        match self {
            SharedShaderUniform::ModelMat => "wuteng_model_mat",
            SharedShaderUniform::ViewMat => "wuteng_view_mat",
            SharedShaderUniform::ProjectionMat => "wuteng_projection_mat",
        }
    }

    /// Returns the shared uniform as a null-terminated [CStr]
    pub const fn as_c_str(self) -> &'static CStr {
        match self {
            SharedShaderUniform::ModelMat => c"wuteng_model_mat",
            SharedShaderUniform::ViewMat => c"wuteng_view_mat",
            SharedShaderUniform::ProjectionMat => c"wuteng_projection_mat",
        }
    }
}
