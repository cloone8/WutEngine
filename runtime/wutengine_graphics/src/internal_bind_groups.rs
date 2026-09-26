//! The engine-provided bind groups for non-user shader parameters, as declared by the built-in `wutengine` shader
//! import

use std::sync::LazyLock;

use wutengine_shadercompiler::engine;

use crate::label;

use super::BindGroup;
use super::BindGroupLayout;

/// Creates the layout of the given engine bind group
fn engine_layout(group: u32) -> BindGroupLayout {
    let bindings = engine::bindings()
        .iter()
        .filter(|binding| binding.group == group)
        .cloned();

    BindGroupLayout::new(label!("Engine group {} layout", group), bindings)
        .expect("Engine bindings must be supported by the runtime")
}

/// Returns the layout for the per-camera bind group
pub fn get_camera_bind_group_layout() -> &'static BindGroupLayout {
    static CAMERA_LAYOUT: LazyLock<BindGroupLayout> =
        LazyLock::new(|| engine_layout(engine::CAMERA_PARAMS_BIND_GROUP_INDEX));

    &CAMERA_LAYOUT
}

/// Creates a new bind group for per-camera parameters
pub fn create_camera_bind_group(name: String) -> BindGroup {
    BindGroup::new(name, get_camera_bind_group_layout())
}

/// Returns the layout for the per-instance bind group
pub fn get_instance_bind_group_layout() -> &'static BindGroupLayout {
    static INSTANCE_LAYOUT: LazyLock<BindGroupLayout> =
        LazyLock::new(|| engine_layout(engine::INSTANCE_PARAMS_BIND_GROUP_INDEX));

    &INSTANCE_LAYOUT
}

/// Creates a new bind group for per-instance parameters
pub fn create_instance_bind_group(name: String) -> BindGroup {
    BindGroup::new(name, get_instance_bind_group_layout())
}
