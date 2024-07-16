use core::{any::Any, fmt::Debug};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct EntityId(u64);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ComponentTypeId(u64);

#[derive(Debug)]
pub struct Entity {
    id: EntityId,
}

pub trait Component: Any {}

pub struct World;

#[derive(Debug)]
pub struct System {
    pub phase: SystemPhase,
    pub func: SystemFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemPhase {
    RuntimeStart,
    Update,
}

#[derive(Debug)]
pub enum SystemFunction {
    Immutable(fn(&World)),
    Mutable(fn(&mut World)),
}
