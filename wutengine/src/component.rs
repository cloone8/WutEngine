//! WutEngine components and component helpers

/// Trait that should be implemented by types that can be
/// used as components in the WutEngine ECS
pub trait Component: Send + Sync + 'static {}
