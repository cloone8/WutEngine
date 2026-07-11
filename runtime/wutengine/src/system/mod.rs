//! WutEngine ECS system registration and query helpers

use alloc::sync::Arc;
use core::{
    any::TypeId,
    fmt::Display,
    sync::atomic::{AtomicU32, Ordering},
};
use std::collections::HashSet;

use rayon::prelude::*;

mod queryable;
mod scheduler;

pub use queryable::*;
use wutengine_util::assert_main_thread;

use crate::{runtime::SystemManifest, world::World};

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
    pending_systems: Option<SystemManifest>,
    current_manifest: SystemManifest,
    by_phase: Vec<(Phase, Vec<SystemSet>)>,
}

impl SystemManager {
    /// Creates a new [SystemManager] without any systems
    pub(crate) fn new() -> Self {
        Self {
            pending_systems: None,
            current_manifest: SystemManifest::empty(),
            by_phase: Vec::new(),
        }
    }

    /// Adds the systems in `manifest` to `self` and updates the schedule
    pub(crate) fn queue_system(&mut self, manifest: SystemManifest) {
        // TODO: Check if the systems are valid? Are they always valid?
        match self.pending_systems.as_mut() {
            Some(pending) => {
                pending.merge(manifest);
            }
            None => {
                self.pending_systems = Some(manifest);
            }
        }
    }

    /// Updates the schedule if any systems were queued with [Self::queue_system]
    pub(crate) fn update_schedule(&mut self) {
        let Some(pending) = self.pending_systems.take() else {
            return;
        };

        if pending.systems.is_empty() {
            return;
        }

        profiling::function_scope!();

        log::info!(
            "Updating system schedule and inserting {} new systems",
            pending.systems.len()
        );

        let mut new_manifest = core::mem::take(&mut self.current_manifest);

        new_manifest.merge(pending);

        self.build_schedule(new_manifest);
    }

    /// Dumps the schedule of the system manager to a string
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

    /// Runs all systems for a given phase, using the provided world
    pub(crate) fn run_systems_for_phase(&self, phase: Phase, world: &crate::world::World) {
        profiling::function_scope!(phase.str());

        assert_main_thread!();

        log::trace!("Running systems for phase {phase}");

        let Some(sets) = self.find_sets_for_phase(phase) else {
            return;
        };

        for set in sets {
            set.systems.par_iter().for_each(|sys| {
                sys(world);
            });
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
    systems: Vec<Arc<GenericSystem>>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// Called once each fixed update. Depends on the configured fixed update time.
    /// Might be any number (or zero) times per frame
    FixedUpdate,

    /// Called once each tick
    Update,

    /// Called once each tick, after the main [Self::Update]
    LateUpdate,

    /// Called after all standard frame logic, right before rendering takes place
    PreRender,
}

impl Phase {
    /// Returns the phase name as a static [str]
    pub(crate) const fn str(self) -> &'static str {
        match self {
            Self::FixedUpdate => "Fixed Update",
            Self::Update => "Update",
            Self::LateUpdate => "Late Update",
            Self::PreRender => "Pre-render",
        }
    }
}

impl Display for Phase {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.str().fmt(f)
    }
}

/// Adds the systems in `manifest` to the main schedule.
///
/// Note that the systems are not inserted immediately, but rather before the next frame phase
#[inline]
pub fn insert_systems(manifest: SystemManifest) {
    crate::runtime::send_to_main_thread(crate::runtime::MainThreadEvent::AddSystem(manifest));
}
