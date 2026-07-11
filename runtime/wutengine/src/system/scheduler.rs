use std::collections::HashSet;

use super::{Phase, SystemManager, SystemSet};
use crate::runtime::{PendingSystem, SystemManifest};

/// Schedule building
impl SystemManager {
    /// Adds the systems in the provided manifest to the schedule of the system manager
    pub(crate) fn build_schedule(&mut self, mut manifest: SystemManifest) {
        log::trace!(
            "Building system schedule from manifest with {} systems",
            manifest.systems.len()
        );

        self.current_manifest = manifest.clone();
        self.by_phase.clear();

        while !manifest.systems.is_empty() {
            let phase = manifest.systems[0].phase;

            self.build_schedule_for_phase(&mut manifest, phase);
        }

        assert!(manifest.systems.is_empty(), "Manifest not empty");

        log::trace!("Done building schedule");
    }

    fn build_schedule_for_phase(&mut self, manifest: &mut SystemManifest, phase: Phase) {
        let mut systems_in_phase = manifest
            .systems
            .extract_if(.., |sys| sys.phase == phase)
            .collect::<Vec<_>>();

        // Systems can only depend on earlier systems, so if we sort from low to high ID we know that all dependencies
        // of the system have already been scheduled
        systems_in_phase.sort_by_key(|pending_system| pending_system.system_id);

        let system_sets =
            if let Some(existing_sets) = self.by_phase.iter_mut().find(|(p, _)| *p == phase) {
                &mut existing_sets.1
            } else {
                let new_set = Vec::new();
                self.by_phase.push((phase, new_set));

                let new_set = self.by_phase.last_mut().unwrap();

                &mut new_set.1
            };

        for system in systems_in_phase {
            Self::insert_system(system_sets, system);
        }
    }

    fn insert_system(sets: &mut Vec<SystemSet>, system: PendingSystem) {
        let mut unsatisfied_dependencies: HashSet<_> =
            HashSet::from_iter(system.dependencies.as_slice());

        let mut set_offset = 0;

        // First we skip forward through the sets until all our dependencies have been satisfied
        while !unsatisfied_dependencies.is_empty() {
            assert!(
                set_offset < sets.len(),
                "Failed to build system schedule because a dependency could not be found"
            );

            for system_id in &sets[set_offset].system_ids {
                unsatisfied_dependencies.remove(system_id);
            }

            set_offset += 1;
        }

        // Now that we've satisfied our dependencies, we check for clashes in the borrows.
        // If we find a clash, we skip to the next set, inserting a new layer if needed
        loop {
            if set_offset >= sets.len() {
                sets.push(SystemSet::new());
            }

            let set = &mut sets[set_offset];

            if !has_clashing_borrows(set, &system) {
                // If we don't have any clashing borrows, we can add the system here.
                // Otherwise we must skip to the next set and try again
                insert_system_into_set(set, system);
                break;
            }

            set_offset += 1;
        }
    }
}

fn insert_system_into_set(set: &mut SystemSet, new_system: PendingSystem) {
    set.system_ids.push(new_system.system_id);
    set.system_names.push(new_system.name);
    set.systems.push(new_system.callback);
    set.shared_borrows.extend(new_system.shared_borrows);
    set.exclusive_borrows.extend(new_system.exclusive_borrows);
}

fn has_clashing_borrows(set: &SystemSet, new_system: &PendingSystem) -> bool {
    // Check if any of the new shared borrows clash with the mutable borrows of the existing set
    for new_shared_borrow in new_system.shared_borrows.iter() {
        if set.exclusive_borrows.contains(new_shared_borrow) {
            return true;
        }
    }

    // Check if any of the new exlusive borrows clash with the mutable or immutable borrows of the existing set
    for new_exclusive_borrow in new_system.exclusive_borrows.iter() {
        if set.shared_borrows.contains(new_exclusive_borrow)
            || set.exclusive_borrows.contains(new_exclusive_borrow)
        {
            return true;
        }
    }

    // No clashes!
    false
}
