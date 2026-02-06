use core::any::TypeId;
use std::collections::HashSet;

use crate::system::{GenericSystem, Phase, Queryable, SystemId};

/// A collection of systems, used during WutEngine runtime initialization to build a
/// system schedule.
///
/// Created with [Self::default], or [Self::empty] if the default systems are not desired
pub struct SystemManifest {
    /// The systems added to the manifest. Not in any particular order
    pub(crate) systems: Vec<PendingSystem>,
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
    {
        self.add_system_with_dependency(phase, name, sys, &[])
    }

    /// Adds a system to the manifest that is dependent on one or more previously inserted systems
    pub fn add_system_with_dependency<Q>(
        &mut self,
        phase: Phase,
        name: Option<&'static str>,
        sys: impl for<'a> Fn(crate::entity::Entity, Q::Item<'a>) + Send + Sync + 'static,
        dependencies: &[SystemId],
    ) -> SystemId
    where
        Q: crate::hecs::Query + Queryable,
    {
        let system_id = SystemId::next(phase);

        for dependency in dependencies {
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

        let callback: Box<GenericSystem> = Box::new(move |world: &crate::world::World| {
            let _name_str = name.unwrap_or("<unnamed system>");

            profiling::scope!("System callback", _name_str);

            let mut query_borrowed = world.ecs.query::<(hecs::Entity, Q)>();

            for (entity, query_return) in query_borrowed.iter() {
                profiling::scope!("System invocation");
                sys(crate::entity::Entity(entity), query_return);
            }
        });

        self.systems.push(PendingSystem {
            name,
            system_id,
            phase,
            shared_borrows,
            exclusive_borrows,
            dependencies: Vec::from(dependencies),
            callback,
        });

        system_id
    }
}

impl Default for SystemManifest {
    fn default() -> Self {
        let mut manifest = Self::empty();

        manifest.add_default_component_systems::<crate::builtins::components::Camera>();

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
