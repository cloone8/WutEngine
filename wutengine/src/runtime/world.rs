//! ECS World management

use core::ops::{Deref, DerefMut};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, RwLock};

use wutengine_util::GlobalManager;

use crate::prelude::Entity;
use crate::profiling;

/// The ECS World manager. Handles Entities, Components, and their lifetimes
pub(crate) struct WorldManager {
    world: RwLock<hecs::World>,
    spawn_queue_send: Sender<hecs::EntityBuilder>,
    spawn_queue_recs: Mutex<Receiver<hecs::EntityBuilder>>,
}

/// The global [WorldManager]
pub(crate) static WORLD_MANAGER: GlobalManager<WorldManager> = GlobalManager::new();

impl WorldManager {
    fn new() -> Self {
        let (send, recv) = channel();

        Self {
            world: RwLock::new(hecs::World::new()),
            spawn_queue_send: send,
            spawn_queue_recs: Mutex::new(recv),
        }
    }

    /// Acquires a shared lock on the [hecs::World] of this manager.
    /// If the world is already exclusively locked, panics instead (as this is a programming error)
    pub(crate) fn shared(&self) -> impl Deref<Target = hecs::World> {
        match self.world.try_read() {
            Ok(lock_guard) => lock_guard,
            Err(std::sync::TryLockError::WouldBlock) => {
                panic!("World already exclusively locked. WutEngine internal error!")
            }
            Err(std::sync::TryLockError::Poisoned(e)) => panic!("World lock poisoned: {e}"),
        }
    }

    /// Acquires an exclusive lock on the [hecs::World] of this manager.
    /// If the world is already shared or exclusively locked, panics instead (as this is a programming error)
    pub(crate) fn exclusive(&self) -> impl DerefMut<Target = hecs::World> {
        match self.world.try_write() {
            Ok(lock_guard) => lock_guard,
            Err(std::sync::TryLockError::WouldBlock) => {
                panic!("World still shared locked. WutEngine internal error!")
            }
            Err(std::sync::TryLockError::Poisoned(e)) => panic!("World lock poisoned: {e}"),
        }
    }

    /// Queues a [hecs::EntityBuilder] for spawning, the next time [run_spawn_queue] is called.
    #[inline]
    pub(crate) fn queue_spawn(&self, entity: hecs::EntityBuilder) {
        self.spawn_queue_send
            .send(entity)
            .expect("Spawn queue dead");
    }
}

/// Initializes the global [WorldManager]
pub(crate) fn init() {
    GlobalManager::init(&WORLD_MANAGER, WorldManager::new());
}

/// Runs the global spawn queue, spawning any queued entities
#[profiling::function]
pub(crate) fn run_spawn_queue() {
    let mut world = WORLD_MANAGER.exclusive();

    let spawn_queue = WORLD_MANAGER.spawn_queue_recs.lock().unwrap();

    for mut builder in spawn_queue.try_iter() {
        let new_entity = Entity::from_hecs(world.spawn(builder.build()));

        log::debug!("Spawned new Entity {new_entity}");
    }
}
