use core::any::{Any, TypeId};
use std::collections::HashMap;

use crate::archetype::{Archetype, ArchetypeId};

#[cfg(test)]
mod test;

mod queries;
pub use queries::*;

use wutengine_core::EntityId;

pub struct World {
    entities: HashMap<EntityId, ArchetypeId>,
    archetypes: HashMap<ArchetypeId, Archetype>,
    type_containers: HashMap<TypeId, Vec<ArchetypeId>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: HashMap::default(),
            archetypes: HashMap::default(),
            type_containers: HashMap::default(),
        }
    }

    pub fn create_entity<T: Any>(&mut self, init_component: T) -> EntityId {
        let id = EntityId::random();
        let initial_archetype = ArchetypeId::new(&[TypeId::of::<T>()]);

        let prev_key = self.entities.insert(id, initial_archetype);

        debug_assert!(prev_key.is_none(), "Entity already present");

        if !self.archetypes.contains_key(&initial_archetype) {
            self.create_single_component_archetype(id, init_component);

            debug_assert!(
                self.archetypes.contains_key(&initial_archetype),
                "Archetype should have been created"
            );
        } else {
            let archetype = self.archetypes.get_mut(&initial_archetype).unwrap();

            let mut component_map = archetype.get_components_for_add(id);
            component_map
                .get_mut(&TypeId::of::<T>())
                .expect("AnyVec could not be found")
                .get_mut()
                .push(init_component);
        }

        id
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        let archetype_id = self.entities.get(&entity).expect("Entity does not exist");
        let archetype = self
            .archetypes
            .get_mut(archetype_id)
            .expect("Could not find archetype");

        let (entity_idx, mut archetype_components) = archetype.get_components_for_remove(entity);

        // First, remove the components belonging to this entity while saving
        // the types of the removed components for later
        let mut removed_archetype_type_ids: Vec<TypeId> = Vec::new();

        for component_vec in archetype_components.values_mut() {
            let borrowed = component_vec.get_mut();

            removed_archetype_type_ids.push(borrowed.inner_type_id());

            borrowed.swap_remove(entity_idx);
        }

        std::mem::drop(archetype_components);

        // If removing the components has left the archetype empty, remove it entirely
        if archetype.is_empty() {
            let old_val = self.archetypes.remove(archetype_id);

            debug_assert!(old_val.is_some());

            // Use the TypeIds we saved before to clean up the type_containers map
            for removed_archetype_type_id in removed_archetype_type_ids.into_iter() {
                if let Some(type_containers) =
                    self.type_containers.get_mut(&removed_archetype_type_id)
                {
                    if let Some(removed_archetype_pos) =
                        type_containers.iter().position(|e| e == archetype_id)
                    {
                        type_containers.swap_remove(removed_archetype_pos);
                    }
                }
            }
        }

        self.entities.remove(&entity);
    }

    fn create_single_component_archetype<T: Any>(&mut self, entity: EntityId, value: T) {
        let new_archetype_id = ArchetypeId::new(&[TypeId::of::<T>()]);

        debug_assert!(
            !self.archetypes.contains_key(&new_archetype_id),
            "Archetype already created"
        );

        let new_archetype = Archetype::new_single(entity, value);

        self.archetypes.insert(new_archetype_id, new_archetype);

        self.type_containers
            .entry(TypeId::of::<T>())
            .or_default()
            .push(new_archetype_id);
    }

    pub fn archetype_ids_for(&self, type_ids: &[TypeId]) -> Vec<ArchetypeId> {
        let mut init_archetypes = self
            .type_containers
            .get(&type_ids[0])
            .expect("Unknown TypeId")
            .clone();

        for type_id in &type_ids[1..] {
            let containing_archetypes = self.type_containers.get(type_id).expect("Unknown TypeId");

            init_archetypes.retain(|e| containing_archetypes.contains(e));
        }

        init_archetypes
    }

    pub unsafe fn query<'a, C: CombinedQuery<'a>>(&'a self, mut callback: impl FnMut(C)) {
        let type_ids = C::get_type_ids();

        assert_unique_type_ids(&type_ids);

        let archetype_ids = self.archetype_ids_for(&type_ids);

        for archetype_id in archetype_ids {
            let archetype = self
                .archetypes
                .get(&archetype_id)
                .expect("Could not find archetype");

            let components = archetype.get_components_for_read(&type_ids);

            C::do_callback(components, &mut callback);
        }
    }
}

#[track_caller]
pub fn assert_unique_type_ids(ids: &[TypeId]) {
    let duplicate = get_first_duplicate_type_id(ids);

    if let Some(duplicate) = duplicate {
        panic!("Duplicate TypeId given: {:?}", duplicate)
    }
}

pub fn get_first_duplicate_type_id(ids: &[TypeId]) -> Option<TypeId> {
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            debug_assert_ne!(i, j);

            if ids[i] == ids[j] {
                return Some(ids[i]);
            }
        }
    }

    None
}

impl World {
    #[track_caller]
    pub fn assert_coherent<const DEBUG_ONLY: bool>(&self) {
        if DEBUG_ONLY && !cfg!(debug_assertions) {
            return;
        }

        // Entity checks
        for (entity_id, containing_archetype_id) in &self.entities {
            let containing_archetype = self.archetypes.get(containing_archetype_id);

            assert!(
                containing_archetype.is_some(),
                "Could not find containing archetype for entity {:?}",
                entity_id
            );

            let containing_archetype = containing_archetype.unwrap();
            containing_archetype.assert_coherent::<DEBUG_ONLY>();

            assert!(
                containing_archetype
                    .get_contained_entities()
                    .iter()
                    .any(|e| e == entity_id),
                "Containing archetype for entity {:?} does not actually contain the entity",
                entity_id
            );
        }

        // Archetype checks
        for (archetype_id, archetype) in &self.archetypes {
            let archetype_types: Vec<TypeId> = archetype.get_contained_types().copied().collect();
            let computed_archetype_id = ArchetypeId::new(&archetype_types);

            assert_eq!(*archetype_id, computed_archetype_id, "Computer archetype ID for archetype does not match its world key. Expected {:?}, computed {:?}", archetype_id, computed_archetype_id);
            assert!(!archetype.is_empty(), "Empty archetype found");
        }

        // Type container checks
        for (type_id, type_id_containers) in &self.type_containers {
            for type_id_container_id in type_id_containers {
                let type_id_container = self.archetypes.get(type_id_container_id);

                assert!(
                    type_id_container.is_some(),
                    "Could not find container for TypeId {:?} and archetype id {:?}",
                    type_id,
                    type_id_container_id
                );

                let type_id_container = type_id_container.unwrap();

                assert!(
                    type_id_container
                        .get_contained_types()
                        .any(|e| e == type_id),
                    "Container for TypeId {:?} does not actually contain the TypeId",
                    type_id
                );
            }
        }
    }
}
