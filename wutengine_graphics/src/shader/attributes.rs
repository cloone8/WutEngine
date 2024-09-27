use core::ffi::CStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedShaderAttribute {
    Position,
}

impl SharedShaderAttribute {
    pub fn as_str(self) -> &'static str {
        match self {
            SharedShaderAttribute::Position => "wuteng_Positions",
        }
    }

    pub fn as_c_str(self) -> &'static CStr {
        match self {
            SharedShaderAttribute::Position => c"wuteng_Positions",
        }
    }
}
