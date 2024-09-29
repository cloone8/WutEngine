use world::World;
use wutengine_core::ReadWriteDescriptor;

pub use wutengine_ecs::world;

use crate::command::Command;

/// A struct describing a callable ECS system and its constraints.
#[derive(Debug)]
pub struct SystemFunctionDescriptor {
    /// The various read/write descriptors of each of the queried components.
    pub read_writes: Vec<ReadWriteDescriptor>,

    /// The actual system function
    pub func: for<'x> fn(&'x World) -> Command,
}

/// A trait implemented by types that are able to produce a system description.
pub trait FunctionDescription {
    /// Returns the description for the system this [FunctionDescription] describes
    fn describe() -> SystemFunctionDescriptor;
}
