use core::ffi::CStr;

use crate::gltypes::GlPosition;
use crate::opengl;
use crate::opengl::types::{GLenum, GLint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderAttribute {
    Position,
}

impl ShaderAttribute {
    pub const ALL: [ShaderAttribute; 1] = [ShaderAttribute::Position];

    #[inline]
    pub const fn as_c_str(self) -> &'static CStr {
        match self {
            ShaderAttribute::Position => c"wuteng_Position",
        }
    }

    #[inline]
    pub fn num_components(self) -> GLint {
        GLint::try_from(match self {
            ShaderAttribute::Position => size_of::<GlPosition>() / size_of::<f32>(),
        })
        .unwrap()
    }

    #[inline]
    pub fn size_bytes(self) -> GLint {
        let base_size = GLint::try_from(match self.component_type() {
            opengl::FLOAT => size_of::<f32>(),
            _ => panic!("Unknown componenet type"),
        })
        .unwrap();

        base_size * self.num_components()
    }

    #[inline]
    pub const fn component_type(self) -> GLenum {
        match self {
            ShaderAttribute::Position => opengl::FLOAT,
        }
    }
}
