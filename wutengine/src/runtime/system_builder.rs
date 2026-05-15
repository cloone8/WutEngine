use core::any::TypeId;
use core::num::NonZero;
use std::collections::HashSet;

use rayon::prelude::*;

use crate::system::{GenericSystem, Phase, Queryable, SystemId};

/// A collection of systems, used during WutEngine runtime initialization to build a
/// system schedule.
///
/// Created with [Self::default], or [Self::empty] if the default systems are not desired
#[derive(Debug)]
pub struct SystemManifest {
    /// The systems added to the manifest. Not in any particular order
    pub(crate) systems: Vec<PendingSystem>,
}

/// A configuration for a system added to a [SystemManifest]
#[derive(Debug)]
pub struct SystemConfig<'a> {
    /// Any dependencies on previously insteded systems
    pub dependencies: &'a [SystemId],

    /// How many query results are processed on a single thread, before the work is split onto another.
    pub parallel_batch_size: Option<NonZero<u32>>,
}

impl<'a> Default for SystemConfig<'a> {
    fn default() -> Self {
        Self {
            dependencies: &[],
            parallel_batch_size: Some(NonZero::new(1024).unwrap()),
        }
    }
}

impl SystemManifest {
    /// Returns an empty [SystemManifest].
    /// This does not include the default engine systems, which
    /// makes WutEngine do nothing by default.
    /// You probably want [SystemManifest::default] instead
    pub const fn empty() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Adds the default systems for the given component, using [crate::component::Component::insert_default_component_systems]
    pub fn add_default_component_systems<C: crate::component::Component>(&mut self) {
        C::insert_default_component_systems(self);
    }

    /// Adds a system to the manifest
    pub fn add_system<Q>(
        &mut self,
        phase: Phase,
        name: Option<&'static str>,
        sys: impl for<'a> Fn(crate::entity::Entity, Q::Item<'a>) + Send + Sync + 'static,
    ) -> SystemId
    where
        Q: crate::hecs::Query + Queryable,
        for<'a> Q::Item<'a>: Send,
    {
        self.add_system_with_config(phase, name, sys, &SystemConfig::default())
    }

    /// Adds a system to the manifest that is dependent on one or more previously inserted systems
    pub fn add_system_with_config<Q>(
        &mut self,
        phase: Phase,
        name: Option<&'static str>,
        sys: impl for<'a> Fn(crate::entity::Entity, Q::Item<'a>) + Send + Sync + 'static,
        config: &SystemConfig,
    ) -> SystemId
    where
        Q: crate::hecs::Query + Queryable,
        for<'a> Q::Item<'a>: Send,
    {
        let system_id = SystemId::next(phase);

        for dependency in config.dependencies {
            assert_eq!(
                phase,
                dependency.phase(),
                "Cannot depend on system in another phase"
            );

            // Not really required due to ordering, but a useful sanity check
            debug_assert!(
                dependency.id() < system_id.id(),
                "Incorrect atomic ordering?"
            );
        }

        let mut shared_borrows = HashSet::with_capacity(Q::NUM_SHARED_BORROWS);
        let mut exclusive_borrows = HashSet::with_capacity(Q::NUM_EXCLUSIVE_BORROWS);

        Q::register_borrows(&mut shared_borrows, &mut exclusive_borrows);

        let batch_size = config.parallel_batch_size;

        let callback: Box<GenericSystem> = Box::new(move |world: &crate::world::World| {
            let _name_str = name.unwrap_or("<unnamed system>");

            profiling::scope!("System callback", _name_str);

            let mut query_borrowed = world.ecs.query::<(hecs::Entity, Q)>();

            if let Some(batch_size) = batch_size {
                // If a parallel batch size was given, we first split the main query
                // into appropriately sized batches
                let par_batches = query_borrowed
                    .iter_batched(batch_size.get())
                    .enumerate()
                    .par_bridge();

                // Here we send the batches to rayon, which automatically distributes them into the thread pool
                par_batches.for_each(|(_i, batch)| {
                    profiling::scope!("System batch", _i.to_string());

                    // Finally, we process the batch on the same thread
                    for (entity, query_return) in batch {
                        profiling::scope!("System invocation");
                        sys(crate::entity::Entity(entity), query_return);
                    }
                });
            } else {
                // If a batch size was not given, we process the batch fully on this thread
                for (entity, query_return) in query_borrowed.iter() {
                    profiling::scope!("System invocation");
                    sys(crate::entity::Entity(entity), query_return);
                }
            };
        });

        self.systems.push(PendingSystem {
            name,
            system_id,
            phase,
            shared_borrows,
            exclusive_borrows,
            dependencies: Vec::from(config.dependencies),
            callback,
        });

        system_id
    }
}

impl Default for SystemManifest {
    fn default() -> Self {
        let mut manifest = Self::empty();

        manifest.add_default_component_systems::<crate::builtins::components::Name>();
        manifest.add_default_component_systems::<crate::builtins::components::rendering::Camera>();
        manifest.add_default_component_systems::<crate::builtins::components::rendering::StaticMeshRenderer>();
        manifest.add_default_component_systems::<crate::builtins::components::rendering::GlobalRenderPass>();

        manifest
    }
}

/// A collection of information on an unscheduled system. Used later
/// for proper scheduling and ordering
#[derive(derive_more::Debug)]
pub(crate) struct PendingSystem {
    /// The system name, if any
    pub(crate) name: Option<&'static str>,

    /// The system ID
    pub(crate) system_id: SystemId,

    /// The phase where the system should run
    pub(crate) phase: Phase,

    /// What component types the system borrows immutably
    pub(crate) shared_borrows: HashSet<TypeId>,

    /// What component types the system borrows mutable
    pub(crate) exclusive_borrows: HashSet<TypeId>,

    /// What dependencies the system has, if any
    pub(crate) dependencies: Vec<SystemId>,

    /// The actual system-running callback
    #[debug(skip)]
    pub(crate) callback: Box<GenericSystem>,
}
