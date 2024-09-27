use core::ffi::CStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedShaderUniform {
    ModelMat,
    ViewMat,
    ProjectionMat,
}

impl SharedShaderUniform {
    pub const fn as_str(self) -> &'static str {
        match self {
            SharedShaderUniform::ModelMat => "wuteng_ModelMat",
            SharedShaderUniform::ViewMat => "wuteng_ViewMat",
            SharedShaderUniform::ProjectionMat => "wuteng_ProjectionMat",
        }
    }

    pub const fn as_c_str(self) -> &'static CStr {
        match self {
            SharedShaderUniform::ModelMat => c"wuteng_ModelMat",
            SharedShaderUniform::ViewMat => c"wuteng_ViewMat",
            SharedShaderUniform::ProjectionMat => c"wuteng_ProjectionMat",
        }
    }
}
