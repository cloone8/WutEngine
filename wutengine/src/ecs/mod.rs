use world::World;
use wutengine_core::ReadWriteDescriptor;

pub use wutengine_ecs::world;

use crate::command::Command;

#[derive(Debug)]
pub struct SystemFunctionDescriptor {
    pub read_writes: Vec<ReadWriteDescriptor>,
    pub func: for<'x> fn(&'x World) -> Command,
}

pub trait FunctionDescription {
    fn describe() -> SystemFunctionDescriptor;
}
