//! WutEngine ECS system registration and query helpers

use core::any::TypeId;
use core::fmt::Display;
use core::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashSet;

mod queryable;
mod scheduler;

pub use queryable::*;

use crate::world::World;

/// The generic type used for a non-typed system callback.
pub(crate) type GenericSystem = dyn Fn(&World) + Send + Sync + 'static;

/// The ID of a system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(u32, Phase);

impl SystemId {
    /// Returns a new SystemId, guaranteed to be higher than any previous ID
    pub(crate) fn next(phase: Phase) -> Self {
        static NEXT_SYSTEM_ID: AtomicU32 = AtomicU32::new(0);

        // Must use AcqRel so we know that systems that were inserted later have a higher ID
        SystemId(NEXT_SYSTEM_ID.fetch_add(1, Ordering::AcqRel), phase)
    }

    /// Returns the raw integer ID of this [SystemId]
    #[inline]
    pub(crate) const fn id(self) -> u32 {
        self.0
    }

    /// Returns the phase to which this system belongs
    #[inline]
    pub const fn phase(self) -> Phase {
        self.1
    }
}

impl PartialOrd for SystemId {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SystemId {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

/// The system manager. Contains the full schedule of all systems, ordered by phase
#[derive(Debug)]
pub(crate) struct SystemManager {
    by_phase: Vec<(Phase, Vec<SystemSet>)>,
}

impl SystemManager {
    /// Creates a new [SystemManager] without any systems
    pub(crate) fn new() -> Self {
        Self {
            by_phase: Vec::new(),
        }
    }

    pub(crate) fn dump(&self) -> String {
        let mut s = String::new();

        for (phase, sets) in &self.by_phase {
            s = format!("{s}Phase: {phase}\n");

            for (stage, set) in sets.iter().enumerate() {
                let set_fmt = set.system_names.join(", ");

                s = format!("{s}\tStage {stage}: {set_fmt}\n");
            }
        }

        s
    }

    pub(crate) fn run_systems_for_phase(&self, phase: Phase, world: &crate::world::World) {
        profiling::function_scope!(phase.str());

        log::trace!("Running systems for phase {phase}");

        let Some(sets) = self.find_sets_for_phase(phase) else {
            return;
        };

        for set in sets {
            //TODO: Parallelize
            for sys in &set.systems {
                sys(world);
            }
        }
    }

    fn find_sets_for_phase(&self, phase: Phase) -> Option<&[SystemSet]> {
        for (set_phase, set) in self.by_phase.iter() {
            if *set_phase == phase {
                return Some(set.as_slice());
            }
        }

        None
    }
}

#[derive(derive_more::Debug)]
struct SystemSet {
    system_names: Vec<&'static str>,
    system_ids: Vec<SystemId>,
    shared_borrows: HashSet<TypeId>,
    exclusive_borrows: HashSet<TypeId>,

    #[debug("{} systems", systems.len())]
    systems: Vec<Box<GenericSystem>>,
}

impl SystemSet {
    fn new() -> Self {
        Self {
            system_names: Vec::new(),
            system_ids: Vec::new(),
            shared_borrows: HashSet::new(),
            exclusive_borrows: HashSet::new(),
            systems: Vec::new(),
        }
    }
}

/// Where, in the process of running a single tick, the system is called
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Called once each fixed update. Depends on the configured fixed update time.
    /// Might be any number (or zero) times per frame
    FixedUpdate,

    /// Called once each tick
    Update,
}

impl Phase {
    /// Returns the phase name as a static [str]
    pub(crate) const fn str(self) -> &'static str {
        match self {
            Self::FixedUpdate => "Fixed Update",
            Self::Update => "Update",
        }
    }
}

impl Display for Phase {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.str().fmt(f)
    }
}
