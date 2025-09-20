//! Types and functions for the "system" part of the WutEngine ECS

use core::any::TypeId;
use core::fmt::Display;
use std::marker::{Send, Sync};
use std::sync::RwLock;

use wutengine_util::hash::nohash_hasher;
use wutengine_util::{GlobalManager, VariantCount};

use rayon::prelude::*;

use crate::profiling;
use crate::system::phase::SystemPhase;

pub mod phase;

pub(crate) struct SystemManager {
    by_phase: RwLock<[Schedule; SystemPhase::VARIANT_COUNT]>,
}

impl SystemManager {
    fn new() -> Self {
        Self {
            by_phase: RwLock::new(core::array::from_fn(|_| Schedule(Vec::new()))),
        }
    }
}

struct Schedule(Vec<Vec<System>>);

struct System {
    exclusive: Vec<TypeId>,
    shared: Vec<TypeId>,
    func: Box<dyn Fn(&hecs::World) + Send + Sync>,
}

pub(crate) static SYSTEM_MANAGER: GlobalManager<SystemManager> = GlobalManager::new();

pub(crate) fn init() {
    GlobalManager::init(&SYSTEM_MANAGER, SystemManager::new());
}

#[profiling::function]
pub(crate) fn run_systems_for_phase(phase: SystemPhase, world: &hecs::World) {
    let phases = SYSTEM_MANAGER.by_phase.read().unwrap();

    let schedule = &phases[phase as u8 as usize];

    for schedule_stage in schedule.0.iter() {
        profiling::scope!("System Stage");

        schedule_stage
            .par_iter()
            .for_each(|system| (system.func)(world));
    }
}

pub fn register_system<
    F: for<'a> Fn(hecs::Entity, Q::Item<'a>) + Sync + Send + 'static,
    Q: hecs::Query,
>(
    sys: F,
    phase: SystemPhase,
) where
    for<'a> <Q as hecs::Query>::Item<'a>: Send,
{
    let mut by_phase = SYSTEM_MANAGER.by_phase.write().unwrap();

    let new_system = System {
        exclusive: Vec::new(),
        shared: Vec::new(),
        func: Box::new(move |world| {
            for (entity, query_result) in world.query::<Q>().into_iter() {
                sys(entity, query_result)
            }
        }),
    };

    by_phase[phase as u8 as usize].0.push(vec![new_system]);
}
