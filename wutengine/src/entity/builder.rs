use crate::prelude::Component;
use crate::runtime::world::WORLD_MANAGER;
use crate::system;

/// A deferred [Entity](super::Entity) builder. Allows composing an entire entity and submitting
/// it for spawning later
pub struct EntityBuilder {
    pub(crate) builder: hecs::EntityBuilder,
    default_system_add_funcs: Vec<fn()>,

    #[cfg(debug_assertions)]
    was_spawned: bool,
}

impl EntityBuilder {
    /// Creates a new [EntityBuilder].
    /// Does nothing unless spawned using [Self::spawn]
    pub fn new() -> Self {
        Self {
            builder: hecs::EntityBuilder::new(),
            default_system_add_funcs: Vec::new(),

            #[cfg(debug_assertions)]
            was_spawned: false,
        }
    }

    /// Adds the given component to this builder, spawning it on this entity once the entity is submitted and spawned
    #[inline]
    pub fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.builder.add(component);

        self.default_system_add_funcs
            .push(system::add_default_systems_for_component::<T>);

        self
    }

    /// Submits the builder and all components that were added to it to the spawn queue. This spawns them in the
    /// game world as soon as possible
    pub fn spawn(mut self) {
        for default_system_add_fn in self.default_system_add_funcs.drain(..) {
            default_system_add_fn();
        }

        WORLD_MANAGER.queue_spawn(core::mem::take(&mut self.builder));

        #[cfg(debug_assertions)]
        {
            self.was_spawned = true;
        }
    }
}

impl Drop for EntityBuilder {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            if !self.was_spawned {
                log::warn!(
                    "EntityBuilder was created but not spawned. Possible user programming error"
                );
            }
        }
    }
}
