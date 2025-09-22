//! Types and functions for the "system" part of the WutEngine ECS

use core::any::TypeId;
use core::fmt::Display;
use std::collections::HashSet;
use std::marker::{Send, Sync};
use std::sync::RwLock;

use wutengine_util::hash::nohash_hasher;
use wutengine_util::{GlobalManager, VariantCount};

use rayon::prelude::*;

use crate::prelude::Component;
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
    log::trace!("Running systems for phase {phase}");

    let phases = SYSTEM_MANAGER.by_phase.read().unwrap();

    let schedule = &phases[phase as u8 as usize];

    for schedule_stage in schedule.0.iter() {
        profiling::scope!("System Stage");

        schedule_stage
            .par_iter()
            .for_each(|system| (system.func)(world));
    }
}

pub fn register_system<Q: hecs::Query + Queryable>(
    sys: impl for<'a> Fn(crate::prelude::Entity, Q::Item<'a>) + Send + Sync + 'static,
    phase: SystemPhase,
) {
    let mut exclusive = HashSet::new();
    let mut shared = HashSet::new();

    Q::register_borrows(&mut shared, &mut exclusive);

    let mut by_phase = SYSTEM_MANAGER.by_phase.write().unwrap();

    let new_system = System {
        exclusive: Vec::from_iter(exclusive),
        shared: Vec::from_iter(shared),
        func: Box::new(move |world| {
            for (entity, query_result) in world.query::<Q>().into_iter() {
                sys(crate::prelude::Entity::from_hecs(entity), query_result);
            }
        }),
    };

    by_phase[phase as u8 as usize].0.push(vec![new_system]);
}

pub trait Queryable {
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>);
}

impl<T> Queryable for &T
where
    T: Component,
{
    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, _exclusive: &mut HashSet<TypeId>) {
        shared.insert(TypeId::of::<T>());
    }
}

impl<T> Queryable for &mut T
where
    T: Component,
{
    #[inline]
    fn register_borrows(_shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        exclusive.insert(TypeId::of::<T>());
    }
}

impl<T> Queryable for Option<T>
where
    T: Queryable,
{
    #[inline]
    fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
        T::register_borrows(shared, exclusive);
    }
}

/// Generates tuple implementations for [Queryable]
macro_rules! queryable_tuples {
    ($t:ident) => {
        impl<$t> Queryable for ($t,)
        where
            $t: Queryable,
        {
            #[inline]
            fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
                $t::register_borrows(shared, exclusive);
            }
        }
    };

    ($t:ident, $($others:ident),*) => {
        impl<$t, $($others),*> Queryable for ($t, $($others),*)
        where
            $t: Queryable,
            $($others: Queryable),*
        {
            #[inline]
            fn register_borrows(shared: &mut HashSet<TypeId>, exclusive: &mut HashSet<TypeId>) {
                $t::register_borrows(shared, exclusive);
                $($others::register_borrows(shared, exclusive));*;
            }
        }

        queryable_tuples!($($others),*);
    };
}

queryable_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
