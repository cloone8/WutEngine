//! Entity management and APIs

use core::fmt::Display;
use core::hash::Hash;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::builtins::components::Name;
use crate::builtins::components::Transform;
use crate::component;
use crate::component::Component;
use crate::runtime::MainThreadEvent;
use crate::runtime::SystemManifest;
use crate::world::World;
use wutengine_util::InitOnce;

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
    profiling::function_scope!();

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

impl Entity {
    /// Spawns a new entity in the game world with an identity rotation and position, and position 0
    #[inline(always)]
    pub fn spawn(name: impl Into<String>) -> Self {
        Self::spawn_impl(name.into(), Some(Transform::new()))
    }

    /// Spawns a new entity in the game world without a transform component, meaning
    /// it has no physical place in the world
    #[inline(always)]
    pub fn spawn_transformless(name: impl Into<String>) -> Self {
        Self::spawn_impl(name.into(), None)
    }

    /// Spawns a new entity in the game world
    #[inline(always)]
    pub fn spawn_at(name: impl Into<String>, transform: Transform) -> Self {
        Self::spawn_impl(name.into(), Some(transform))
    }

    #[inline]
    fn spawn_impl(name: String, transform: Option<Transform>) -> Self {
        let id = crate::world::get_world().ecs.reserve_entity();

        let name = Name::new(name);
        let new_entity = Entity(id);

        log::debug!("Spawning new entity {name} ({new_entity})");

        let mut new_entity = new_entity.add_component(name);

        if let Some(transform) = transform {
            new_entity = new_entity.add_component(transform);
        }

        new_entity
    }

    /// Destroys an entity and removes its components
    pub fn destroy(self) {
        let entity = self;
        log::debug!("Destroying entity {entity}");

        ENTITY_QUEUES
            .destroy_entities_queue
            .send(entity)
            .expect("Runtime stopped");
    }

    /// Adds a new component to the given entity.
    /// The component is not actually inserted into the world immediately, and is instead processed
    /// right before the next frame-phase callback.
    pub fn add_component<C: Component>(self, component: C) -> Self {
        let mut builder = hecs::EntityBuilder::new();

        builder.add(component);

        if component::should_insert_default_component_systems::<C>() {
            log::debug!(
                "Adding default component systems for {}",
                core::any::type_name::<C>()
            );

            let mut manifest = SystemManifest::empty();

            C::insert_default_component_systems(&mut manifest);

            crate::runtime::send_to_main_thread(MainThreadEvent::AddSystem(manifest));
        }

        ENTITY_QUEUES
            .new_component_queue
            .send((self, builder))
            .expect("Runtime stopped");

        self
    }
}
