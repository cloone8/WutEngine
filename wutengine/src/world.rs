//! World management for the WutEngine runtime

use core::ops::{Deref, DerefMut};
use std::sync::RwLock;

use crate::util::InitOnce;

static WORLD: InitOnce<RwLock<World>> = InitOnce::new();

/// Initializes the global "world", which contains the ECS data
pub(crate) fn initialize() {
    log::trace!("Initializing world");

    InitOnce::init(&WORLD, RwLock::new(World::new()));
}

/// Returns a type that dereferences to a [World].
/// Might be a lock guard, so drop as soon as possible
#[inline(always)]
pub(crate) fn get_world() -> impl Deref<Target = World> {
    WORLD.read().unwrap()
}

/// Returns a type that mutably dereferences to a [World].
/// Might be a lock guard, so drop as soon as possible
#[inline(always)]
pub(crate) fn get_world_mut() -> impl DerefMut<Target = World> {
    WORLD.write().unwrap()
}

/// Manager of all entities and components currently living in the game engine
pub(crate) struct World {
    /// The raw [hecs] world
    pub(crate) ecs: hecs::World,
}

impl World {
    /// Creates a new, empty, [World]
    pub(crate) fn new() -> Self {
        Self {
            ecs: hecs::World::new(),
        }
    }
}
