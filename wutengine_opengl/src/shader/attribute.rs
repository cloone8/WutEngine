//! TODO: Migrate to shared [wutengine_graphics::shader::attributes] module

use core::ffi::CStr;

use crate::gltypes::GlVec3f;
use crate::opengl;
use crate::opengl::types::{GLenum, GLint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShaderAttribute {
    Position,
}

impl ShaderAttribute {
    pub(crate) const ALL: [ShaderAttribute; 1] = [ShaderAttribute::Position];

    #[inline]
    pub(crate) const fn as_c_str(self) -> &'static CStr {
        match self {
            ShaderAttribute::Position => c"wuteng_Position",
        }
    }

    #[inline]
    pub(crate) fn num_components(self) -> GLint {
        GLint::try_from(match self {
            ShaderAttribute::Position => size_of::<GlVec3f>() / size_of::<f32>(),
        })
        .unwrap()
    }

    #[inline]
    pub(crate) fn size_bytes(self) -> GLint {
        let base_size = GLint::try_from(match self.component_type() {
            opengl::FLOAT => size_of::<f32>(),
            _ => panic!("Unknown componenet type"),
        })
        .unwrap();

        base_size * self.num_components()
    }

    #[inline]
    pub(crate) const fn component_type(self) -> GLenum {
        match self {
            ShaderAttribute::Position => opengl::FLOAT,
        }
    }
}
