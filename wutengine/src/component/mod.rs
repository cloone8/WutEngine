//! Types and functions for the `component` part of the WutEngine ECS

use core::any::Any;

pub use hecs::Component as Queryable;

/// A component, able to be attached to an [Entity](crate::entity::Entity)
pub trait Component: Any + Send + Sync {
    /// Use this function to lazily register systems for this component (with [crate::system::register_system]),
    /// when an instance of the component type is first added to the engine runtime.
    fn add_default_systems()
    where
        Self: Sized,
    {
    }
}
