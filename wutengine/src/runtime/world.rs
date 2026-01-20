//! World management for the WutEngine runtime

/// Manager of all entities and components currently living in the game engine
pub(crate) struct World {
    ecs: hecs::World,
}

impl World {
    /// Creates a new, empty, [World]
    pub(crate) fn new() -> Self {
        Self {
            ecs: hecs::World::new(),
        }
    }
}
