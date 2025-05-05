//! Shader uniform builtins

use bitflags::{Flags, bitflags};

use super::uniform::SingleUniformBinding;

bitflags! {
    /// The set of builtin uniforms a shader uses
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ShaderBuiltins: u32 {
        /// The viewport constants
        const VIEWPORT_CONSTS = 0b00000000000000000000000000000001;

        /// The instance constants
        const INSTANCE_CONSTS = 0b00000000000000000000000000000010;
    }
}

impl ShaderBuiltins {
    /// Returns the binding for this [ShaderBuiltins], assuming it is
    /// a single flag. Panics if there are multiple flags
    pub fn binding(self) -> SingleUniformBinding {
        assert!(!self.contains_unknown_bits());
        assert_eq!(
            1,
            self.bits().count_ones(),
            "Multiple bits set, cannot return one binding"
        );

        match self {
            ShaderBuiltins::VIEWPORT_CONSTS => SingleUniformBinding {
                name: "wuteng_vp_const_block".to_string(),
                group: 0,
                binding: 0,
            },
            ShaderBuiltins::INSTANCE_CONSTS => SingleUniformBinding {
                name: "wuteng_instance_const_block".to_string(),
                group: 0,
                binding: 1,
            },
            _ => unreachable!("Should be checked at the start of the function"),
        }
    }
}
