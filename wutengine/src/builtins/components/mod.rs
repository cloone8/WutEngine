//! Builtin components

mod name;
pub mod rendering;
mod transform;

pub use name::*;
pub use transform::*;

use crate::runtime::SystemManifest;

/// Registers all systems for the built-in components
pub fn register_builtin_component_systems(manifest: &mut SystemManifest) {
    manifest.add_default_component_systems::<Name>();
    manifest.add_default_component_systems::<Transform>();
    manifest.add_default_component_systems::<rendering::Camera>();
    manifest.add_default_component_systems::<rendering::StaticMeshRenderer>();
    manifest.add_default_component_systems::<rendering::GlobalRenderPass>();
}
