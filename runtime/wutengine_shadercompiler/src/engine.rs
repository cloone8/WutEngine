//! The engine's bind group layout, and the built-in `wutengine` import that exposes it to shaders

use std::sync::LazyLock;

use wutengine_assets::assets::shader::Binding;

/// Group index of the per-camera bind group
pub const CAMERA_PARAMS_BIND_GROUP_INDEX: u32 = 0;

/// Group index of the material bind group
pub const MATERIAL_PARAMS_BIND_GROUP_INDEX: u32 = 1;

/// Group index of the per-instance bind group
pub const INSTANCE_PARAMS_BIND_GROUP_INDEX: u32 = 2;

/// Number of bind groups a shader can use
pub const NUM_BIND_GROUPS: u32 = 3;

/// The name of the built-in import, as in `#import "wutengine"`
pub const IMPORT_NAME: &str = "wutengine";

/// The source of the built-in import: the group index constants, and the camera and instance bindings
pub fn import_source() -> &'static str {
    static SOURCE: LazyLock<String> = LazyLock::new(|| {
        format!(
            "const WUTENGINE_CAMERA_GROUP: u32 = {CAMERA_PARAMS_BIND_GROUP_INDEX}u;\n\
             const WUTENGINE_MATERIAL_GROUP: u32 = {MATERIAL_PARAMS_BIND_GROUP_INDEX}u;\n\
             const WUTENGINE_INSTANCE_GROUP: u32 = {INSTANCE_PARAMS_BIND_GROUP_INDEX}u;\n\
             {}",
            include_str!("wutengine.wgsl")
        )
    });

    &SOURCE
}

/// The bindings the engine provides in the camera and instance groups, as declared by [`import_source`]
pub fn bindings() -> &'static [Binding] {
    static BINDINGS: LazyLock<Vec<Binding>> = LazyLock::new(|| {
        let module = naga::front::wgsl::parse_str(import_source())
            .expect("Built-in wutengine import is invalid WGSL");

        crate::bindings::find_bindings(&module, |_| true)
            .expect("Built-in wutengine import has bad bindings")
    });

    &BINDINGS
}

/// A shader binding that conflicts with the engine's bind group layout
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum LayoutErr {
    /// A binding in an engine group that isn't the engine's
    #[display(
        "`{}` is in group {}, which is reserved for the engine. Use `#import \"{IMPORT_NAME}\"` and \
         @group(WUTENGINE_MATERIAL_GROUP) for material parameters",
        _0.name,
        _0.group
    )]
    Reserved(#[error(not(source))] Box<Binding>),

    /// A binding in a group the engine doesn't have
    #[display(
        "`{}` is in group {}, but shaders can only use groups 0 to {}",
        _0.name,
        _0.group,
        NUM_BIND_GROUPS - 1
    )]
    UnknownGroup(#[error(not(source))] Box<Binding>),
}

/// Checks that `shader_bindings` fit the engine layout: engine groups hold exactly the engine's bindings
pub fn check_layout(shader_bindings: &[Binding]) -> Result<(), LayoutErr> {
    for binding in shader_bindings {
        match binding.group {
            MATERIAL_PARAMS_BIND_GROUP_INDEX => {}
            CAMERA_PARAMS_BIND_GROUP_INDEX | INSTANCE_PARAMS_BIND_GROUP_INDEX => {
                if !bindings().contains(binding) {
                    return Err(LayoutErr::Reserved(Box::new(binding.clone())));
                }
            }
            _ => return Err(LayoutErr::UnknownGroup(Box::new(binding.clone()))),
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    /// Tests that the built-in import parses and lands in the engine groups, to prevent a broken import from only
    /// failing at runtime
    #[test]
    fn engine_bindings() {
        let bindings = bindings();

        assert!(
            bindings
                .iter()
                .any(|b| b.group == CAMERA_PARAMS_BIND_GROUP_INDEX)
        );
        assert!(
            bindings
                .iter()
                .any(|b| b.group == INSTANCE_PARAMS_BIND_GROUP_INDEX)
        );
        check_layout(bindings).unwrap();
    }
}
