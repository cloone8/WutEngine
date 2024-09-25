use core::any::{Any, TypeId};
use std::collections::HashMap;

use crate::archetype::{self, Archetype, ArchetypeId, TypeDescriptorSet};
use crate::vec::AnyVecStorageDescriptor;

#[cfg(test)]
mod test;

mod checks;
mod queries;

pub use queries::*;

use wutengine_core::{Component, EntityId};

#[derive(Debug, Default)]
pub struct World {
    entities: HashMap<EntityId, Option<ArchetypeId>>,
    archetypes: HashMap<ArchetypeId, Archetype>,
    type_containers: HashMap<TypeId, Vec<ArchetypeId>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_entity(&mut self) -> EntityId {
        let id = EntityId::random();

        let prev_key = self.entities.insert(id, None);

        debug_assert!(prev_key.is_none(), "Entity already present");

        id
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        let archetype_id = self.entities.get(&entity).expect("Entity does not exist");

        if archetype_id.is_none() {
            self.entities.remove(&entity);
            return;
        }

        let archetype_id = archetype_id.as_ref().unwrap();

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
            let to_delete = *archetype_id;
            self.delete_archetype(&to_delete);
        }

        self.entities.remove(&entity);
    }

    /// Deletes the given archetype. The archetype must already be empty.
    /// The types contained
    fn delete_archetype(&mut self, id: &ArchetypeId) {
        let archetype = self
            .archetypes
            .get(id)
            .expect("Cannot find archetype to remove");

        assert!(archetype.is_empty(), "Archetype is not empty");

        let archetype_types: Vec<TypeId> = archetype.get_contained_types().copied().collect();

        self.archetypes.remove(id);

        for removed_archetype_type_id in archetype_types.into_iter() {
            if let Some(type_containers) = self.type_containers.get_mut(&removed_archetype_type_id)
            {
                if let Some(removed_archetype_pos) = type_containers.iter().position(|e| e == id) {
                    type_containers.swap_remove(removed_archetype_pos);
                }
            }
        }
    }

    pub fn add_component_to_entity<T: Any>(&mut self, entity: EntityId, component: T) {
        let current_archetype_id = *self.entities.get(&entity).expect("Entity not found");

        if current_archetype_id.is_none() {
            let initial_archetype = ArchetypeId::new(&[TypeId::of::<T>()]);

            if !self.archetypes.contains_key(&initial_archetype) {
                self.create_single_component_archetype(entity, component);

                debug_assert!(
                    self.archetypes.contains_key(&initial_archetype),
                    "Archetype should have been created"
                );
            } else {
                let archetype = self.archetypes.get_mut(&initial_archetype).unwrap();

                let mut component_map = archetype.get_components_for_add(entity);
                component_map
                    .get_mut(&TypeId::of::<T>())
                    .expect("AnyVec could not be found")
                    .get_mut()
                    .push(component);
            }

            self.entities.insert(entity, Some(initial_archetype));

            return;
        }

        let current_archetype_id = current_archetype_id.unwrap();

        let current_archetype = self
            .archetypes
            .get(&current_archetype_id)
            .expect("Archetype not found");
        let mut typeid_set = Vec::from_iter(current_archetype.get_contained_types().copied());

        let new_component_type = TypeId::of::<T>();

        assert!(!typeid_set.contains(&new_component_type));

        typeid_set.push(new_component_type);

        let new_archetype_id = ArchetypeId::new(&typeid_set);

        // Ensure target archetype exists first
        #[expect(
            clippy::map_entry,
            reason = "Clippy's solution causes borrow-checker problems"
        )]
        if !self.archetypes.contains_key(&new_archetype_id) {
            let new_empty_archetype =
                Archetype::new_from_template(current_archetype, TypeDescriptorSet::new::<T>(), &[]);

            for type_id in new_empty_archetype.get_contained_types() {
                self.type_containers
                    .entry(*type_id)
                    .or_default()
                    .push(new_archetype_id);
            }

            self.archetypes
                .insert(new_archetype_id, new_empty_archetype);
        }

        // Scary reference magic: Rust does not allow multiple mutable borrows to the
        // same hashmap, so we trick the borrowchecker by casting them to pointers.
        // To ensure this is valid, we must ensure that we do not mutably borrow the hashmap
        // again while holding these pointers
        let source_archetype_ptr =
            self.archetypes.get_mut(&current_archetype_id).unwrap() as *mut Archetype;
        let target_archetype_ptr =
            self.archetypes.get_mut(&new_archetype_id).unwrap() as *mut Archetype;

        assert_ne!(
            source_archetype_ptr, target_archetype_ptr,
            "current is target, invalid aliasing"
        );

        // SAFETY: Pointers are valid as long as the hashmap containing them is not modified, as we
        // know they do not reference the same archetype
        let source_archetype_empty = unsafe {
            source_archetype_ptr
                .as_mut()
                .unwrap()
                .move_entity_to::<false>(entity, target_archetype_ptr.as_mut().unwrap());

            target_archetype_ptr
                .as_mut()
                .unwrap()
                .add_to_entity_unchecked(entity, component);

            source_archetype_ptr.as_ref().unwrap().is_empty()
        };

        if source_archetype_empty {
            self.delete_archetype(&current_archetype_id);
        }

        let old_archetype_id = self.entities.insert(entity, Some(new_archetype_id));

        debug_assert!(old_archetype_id.is_some());
        debug_assert_eq!(current_archetype_id, old_archetype_id.unwrap().unwrap());

        self.assert_coherent::<true>();
    }

    pub fn remove_components_from_entity(&mut self, entity: EntityId, components: &[TypeId]) {
        let current_archetype_id = *self.entities.get(&entity).expect("Entity not found");

        if current_archetype_id.is_none() {
            //TODO: Don't delete the entity by default if its last component is deleted
            return;
        }

        let current_archetype_id = current_archetype_id.unwrap();

        let current_archetype = self
            .archetypes
            .get(&current_archetype_id)
            .expect("Archetype not found");

        let new_typeid_set = Vec::from_iter(
            current_archetype
                .get_contained_types()
                .filter(|t_id| !components.contains(t_id))
                .copied(),
        );

        if new_typeid_set.is_empty() {
            self.remove_entity(entity);
            return;
        }

        let new_archetype_id = ArchetypeId::new(&new_typeid_set);

        // Ensure target archetype exists first
        #[expect(
            clippy::map_entry,
            reason = "Clippy's solution causes borrow-checker problems"
        )]
        if !self.archetypes.contains_key(&new_archetype_id) {
            let new_empty_archetype = Archetype::new_from_template(
                current_archetype,
                TypeDescriptorSet::new_empty(),
                components,
            );

            for type_id in new_empty_archetype.get_contained_types() {
                self.type_containers
                    .entry(*type_id)
                    .or_default()
                    .push(new_archetype_id);
            }

            self.archetypes
                .insert(new_archetype_id, new_empty_archetype);
        }

        // Scary reference magic: Rust does not allow multiple mutable borrows to the
        // same hashmap, so we trick the borrowchecker by casting them to pointers.
        // To ensure this is valid, we must ensure that we do not mutably borrow the hashmap
        // again while holding these pointers
        let source_archetype_ptr =
            self.archetypes.get_mut(&current_archetype_id).unwrap() as *mut Archetype;
        let target_archetype_ptr =
            self.archetypes.get_mut(&new_archetype_id).unwrap() as *mut Archetype;

        assert_ne!(
            source_archetype_ptr, target_archetype_ptr,
            "current is target, invalid aliasing"
        );

        // SAFETY: Pointers are valid as long as the hashmap containing them is not modified, as we
        // know they do not reference the same archetype
        let source_archetype_empty = unsafe {
            source_archetype_ptr
                .as_mut()
                .unwrap()
                .move_entity_to::<true>(entity, target_archetype_ptr.as_mut().unwrap());

            source_archetype_ptr.as_ref().unwrap().is_empty()
        };

        if source_archetype_empty {
            self.delete_archetype(&current_archetype_id);
        }

        let old_archetype_id = self.entities.insert(entity, Some(new_archetype_id));

        debug_assert!(old_archetype_id.is_some());
        debug_assert_eq!(current_archetype_id, old_archetype_id.unwrap().unwrap());

        self.assert_coherent::<true>();
    }

    fn create_single_component_archetype<T: Any>(&mut self, entity: EntityId, component: T) {
        let new_archetype_id = ArchetypeId::new(&[TypeId::of::<T>()]);

        debug_assert!(
            !self.archetypes.contains_key(&new_archetype_id),
            "Archetype already created"
        );

        let new_archetype = Archetype::new_single(entity, component);

        self.archetypes.insert(new_archetype_id, new_archetype);

        self.type_containers
            .entry(TypeId::of::<T>())
            .or_default()
            .push(new_archetype_id);
    }
}
