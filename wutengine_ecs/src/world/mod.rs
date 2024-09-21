use core::any::{Any, TypeId};
use std::collections::HashMap;

use crate::archetype::{Archetype, ArchetypeId};
use crate::vec::AnyVecStorageDescriptor;

#[cfg(test)]
mod test;

mod queries;
pub use queries::*;

use wutengine_core::EntityId;

#[derive(Debug, Default)]
pub struct World {
    entities: HashMap<EntityId, ArchetypeId>,
    archetypes: HashMap<ArchetypeId, Archetype>,
    type_containers: HashMap<TypeId, Vec<ArchetypeId>>,
}

impl World {
    pub fn new() -> Self {
        Self::default()
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

        let old_archetype_id = self.entities.insert(entity, new_archetype_id);

        debug_assert!(old_archetype_id.is_some());
        debug_assert_eq!(current_archetype_id, old_archetype_id.unwrap());

        self.assert_coherent::<true>();
    }

    pub fn remove_components_from_entity(&mut self, entity: EntityId, components: &[TypeId]) {
        let current_archetype_id = *self.entities.get(&entity).expect("Entity not found");
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

    /// Queries the world
    ///
    /// # Safety
    ///
    /// This function mutably borrows using unsafe. To ensure safety, the following
    /// rule must be upheld:
    ///
    /// For any mutably queried type `T`, no other queries must be running that either
    /// mutably or immutably borrow `T`
    pub unsafe fn query<'a, C, F, O>(&'a self, callback: F) -> Vec<O>
    where
        C: CombinedQuery<'a>,
        F: Fn(EntityId, C) -> O,
    {
        let type_ids = C::get_type_ids();

        assert_unique_type_ids(&type_ids);

        let archetype_ids = self.archetype_ids_for(&type_ids);

        let mut output = Vec::new();

        for archetype_id in archetype_ids {
            let archetype = self
                .archetypes
                .get(&archetype_id)
                .expect("Could not find archetype");

            let components = archetype.get_components_for_read(&type_ids);
            let entities = archetype.get_contained_entities();

            output.extend(C::do_callback(entities, components, &callback));
        }

        output
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

#[derive(Debug, Clone)]
pub(crate) struct TypeDescriptorSet {
    pub descriptors: Vec<(TypeId, AnyVecStorageDescriptor)>,
}

impl TypeDescriptorSet {
    pub fn new_empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }
    pub fn new<T: Any>() -> Self {
        Self {
            descriptors: vec![(TypeId::of::<T>(), AnyVecStorageDescriptor::new::<T>())],
        }
    }

    pub fn add<T: Any>(&mut self) {
        self.descriptors
            .push((TypeId::of::<T>(), AnyVecStorageDescriptor::new::<T>()));
    }
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
