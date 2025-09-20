//! Types and functions for the `component` part of the WutEngine ECS

use core::any::Any;

pub use hecs::Component as Queryable;

/// A component, able to be attached to an [Entity](crate::entity::Entity)
pub trait Component: Any + Send + Sync {}
