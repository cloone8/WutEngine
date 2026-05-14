//! WutEngine components and component helpers

/// Trait that should be implemented by types that can be
/// used as components in the WutEngine ECS
pub trait Component: Send + Sync + Default + 'static {
    /// Adds the systems that are always used by this component into the given manifest.
    ///
    /// Optional usability helper
    fn insert_default_component_systems(_manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
    }
}
