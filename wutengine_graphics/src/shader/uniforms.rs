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
            SharedShaderUniform::ModelMat => "wuteng_ModelMat",
            SharedShaderUniform::ViewMat => "wuteng_ViewMat",
            SharedShaderUniform::ProjectionMat => "wuteng_ProjectionMat",
        }
    }

    /// Returns the shared uniform as a null-terminated [CStr]
    pub const fn as_c_str(self) -> &'static CStr {
        match self {
            SharedShaderUniform::ModelMat => c"wuteng_ModelMat",
            SharedShaderUniform::ViewMat => c"wuteng_ViewMat",
            SharedShaderUniform::ProjectionMat => c"wuteng_ProjectionMat",
        }
    }
}
