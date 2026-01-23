//! Entity management and APIs

use core::fmt::Display;
use core::hash::Hash;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::component::Component;
use crate::util::InitOnce;
use crate::world::World;

static ENTITY_QUEUES: InitOnce<EntityCommandQueues> = InitOnce::new();

/// Utility struct containing the global entity command queues
#[derive(Debug)]
struct EntityCommandQueues {
    new_component_queue: Sender<(Entity, hecs::EntityBuilder)>,
    destroy_entities_queue: Sender<Entity>,
}

/// The entity manager
#[derive(Debug)]
pub(crate) struct EntityManager {
    new_components: Receiver<(Entity, hecs::EntityBuilder)>,
    entities_to_destroy: Receiver<Entity>,
}

/// Initializes the global entity command queues and returns
/// the entity manager
pub(crate) fn initialize() -> EntityManager {
    let (new_components_send, new_components_recv) = channel::<(Entity, hecs::EntityBuilder)>();
    let (entities_to_destroy_send, entites_to_destroy_recv) = channel::<Entity>();

    let entity_command_queues = EntityCommandQueues {
        new_component_queue: new_components_send,
        destroy_entities_queue: entities_to_destroy_send,
    };

    InitOnce::init(&ENTITY_QUEUES, entity_command_queues);

    EntityManager {
        new_components: new_components_recv,
        entities_to_destroy: entites_to_destroy_recv,
    }
}

/// Processes the entity changes queued in `manager`, into `world`
pub(crate) fn process_changes(world: &mut World, manager: &EntityManager) {
    log::trace!("Processing entity manager changes");

    // First we make sure all reserved entities are inserted
    world.ecs.flush();

    // Now we add all new components
    let mut num_added = 0;

    for (entity, mut new_component_builder) in manager.new_components.try_iter() {
        if let Err(hecs::NoSuchEntity) = world.ecs.insert(entity.0, new_component_builder.build()) {
            log::error!(
                "Failed to insert components on entity {entity} because it does not exist in the world"
            );
            continue;
        }

        num_added += 1;
    }

    if num_added > 0 {
        log::debug!("Added {num_added} new components");
    }

    let mut num_destroyed = 0;

    for entity in manager.entities_to_destroy.try_iter() {
        if let Err(hecs::NoSuchEntity) = world.ecs.despawn(entity.0) {
            log::error!("Failed to destroy entity {entity} because it does not exist in the world");
            continue;
        }

        num_destroyed += 1;
    }

    if num_destroyed > 0 {
        log::debug!("Destroyed {num_destroyed} entities");
    }
}

/// The ID of a WutEngine entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity(pub(crate) hecs::Entity);

impl Display for Entity {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:016x}", self.0.to_bits().get())
    }
}

impl Hash for Entity {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0.to_bits().get())
    }
}

impl nohash_hasher::IsEnabled for Entity {}

/// Proxy APIs for usability purposes
impl Entity {
    /// See [spawn]
    #[inline(always)]
    pub fn spawn() -> Self {
        spawn()
    }

    /// See [destroy]
    #[inline(always)]
    pub fn destroy(self) {
        destroy(self)
    }

    /// See [add_component]
    #[inline(always)]
    pub fn add_component(self, component: impl Component) {
        add_component(self, component)
    }
}

/// Spawns a new entity in the game world
pub fn spawn() -> Entity {
    let id = crate::world::get_world().ecs.reserve_entity();

    let new_entity = Entity(id);

    log::debug!("Spawning new entity {new_entity}");

    new_entity
}

/// Destroys an entity and removes its components
pub fn destroy(entity: Entity) {
    log::debug!("Destroying entity {entity}");

    ENTITY_QUEUES
        .destroy_entities_queue
        .send(entity)
        .expect("Runtime stopped");
}

/// Adds a new component to the given entity.
/// The component is not actually inserted into the world immediately, and is instead processed
/// right before the next frame-phase callback.
pub fn add_component(entity: Entity, component: impl Component) {
    let mut builder = hecs::EntityBuilder::new();

    builder.add(component);

    ENTITY_QUEUES
        .new_component_queue
        .send((entity, builder))
        .expect("Runtime stopped");
}
