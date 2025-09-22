use crate::prelude::Component;
use crate::runtime::world::WORLD_MANAGER;

pub struct EntityBuilder {
    pub(crate) builder: hecs::EntityBuilder,

    #[cfg(debug_assertions)]
    was_spawned: bool,
}

impl EntityBuilder {
    pub(crate) fn new() -> Self {
        Self {
            builder: hecs::EntityBuilder::new(),
            was_spawned: false,
        }
    }

    #[inline]
    pub fn add_component<T: Component>(&mut self, component: T) -> &mut Self {
        self.builder.add(component);
        self
    }

    pub fn spawn(mut self) {
        WORLD_MANAGER.queue_spawn(core::mem::take(&mut self.builder));
        self.was_spawned = true;
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
