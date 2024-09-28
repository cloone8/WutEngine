use core::any::TypeId;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use std::collections::HashMap;

use crate::vec::{AnyVec, Dynamic};

mod archetype_id;
mod descriptorset;

pub(crate) use archetype_id::*;
pub(crate) use descriptorset::*;
use wutengine_core::EntityId;

#[derive(Debug)]
pub(crate) struct Archetype {
    entities: Vec<EntityId>,
    components: HashMap<TypeId, UnsafeCell<AnyVec>>,
}

#[derive(Debug)]
pub(crate) struct ArchetypeMapMut<'a> {
    inner: &'a mut Archetype,
}

impl<'a> ArchetypeMapMut<'a> {
    pub(crate) fn new(archetype: &'a mut Archetype) -> Self {
        Self { inner: archetype }
    }
}

impl<'a> Deref for ArchetypeMapMut<'a> {
    type Target = HashMap<TypeId, UnsafeCell<AnyVec>>;

    fn deref(&self) -> &Self::Target {
        &self.inner.components
    }
}

impl<'a> DerefMut for ArchetypeMapMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner.components
    }
}

impl<'a> Drop for ArchetypeMapMut<'a> {
    #[track_caller]
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.inner.assert_coherent::<true>();
        }
    }
}

impl Archetype {
    fn mutmap(&mut self) -> ArchetypeMapMut {
        ArchetypeMapMut::new(self)
    }

    pub(crate) fn new_single(entity: EntityId, value: Dynamic) -> Self {
        let component_type = value.inner_type();
        let component_vec = value.add_to_new_vec();

        let mut components_map = HashMap::default();
        components_map.insert(component_type, UnsafeCell::new(component_vec));

        Archetype {
            entities: vec![entity],
            components: components_map,
        }
    }

    pub(crate) fn new_from_template(
        template: &Archetype,
        extra_types: TypeDescriptorSet,
        filtered_types: &[TypeId],
    ) -> Self {
        let mut new_components = HashMap::<TypeId, UnsafeCell<AnyVec>>::default();

        for (typeid, storagecell) in &template.components {
            if filtered_types.contains(typeid) {
                continue;
            }

            new_components.insert(
                *typeid,
                UnsafeCell::new(AnyVec::from_descriptor(unsafe {
                    storagecell.get().as_ref().unwrap().get_descriptor()
                })),
            );
        }

        for (extra_type_id, extra_storage_desc) in extra_types.descriptors {
            assert!(!new_components.contains_key(&extra_type_id));
            assert!(!filtered_types.contains(&extra_type_id));

            new_components.insert(
                extra_type_id,
                UnsafeCell::new(AnyVec::from_descriptor(extra_storage_desc)),
            );
        }

        Self {
            entities: Vec::new(),
            components: new_components,
        }
    }

    pub(crate) fn get_contained_entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub(crate) fn get_contained_types(&self) -> impl Iterator<Item = &TypeId> {
        self.components.keys()
    }

    pub(crate) fn get_components_for_read(
        &self,
        types: &[TypeId],
    ) -> Vec<Option<&UnsafeCell<AnyVec>>> {
        types.iter().map(|t| self.components.get(t)).collect()
    }

    pub(crate) fn get_components_for_add(&mut self, to_add: EntityId) -> ArchetypeMapMut {
        debug_assert!(
            !self.entities.contains(&to_add),
            "Entity already present in archetype: {:?}",
            to_add
        );

        self.entities.push(to_add);

        self.mutmap()
    }

    pub(crate) fn get_components_for_remove(
        &mut self,
        to_remove: EntityId,
    ) -> (usize, ArchetypeMapMut) {
        let entity_idx = self
            .entities
            .iter()
            .position(|e| *e == to_remove)
            .unwrap_or_else(|| panic!("Entity not present in archetype: {:?}", to_remove));

        self.entities.swap_remove(entity_idx);

        (entity_idx, self.mutmap())
    }

    pub(crate) fn move_entity_to<const ALLOW_REMOVAL: bool>(
        &mut self,
        to_move: EntityId,
        destination: &mut Self,
    ) {
        let entity_idx = self
            .entities
            .iter()
            .position(|e| *e == to_move)
            .expect("Entity to-move not found in source archetype");

        let removed_entity = self.entities.swap_remove(entity_idx);

        debug_assert_eq!(to_move, removed_entity, "Removed the wrong entity");

        for (type_id, storage) in &mut self.components {
            let target_storage_opt = destination.components.get_mut(type_id);

            if ALLOW_REMOVAL && target_storage_opt.is_none() {
                // If we allow removals, then we simply drop
                // the component if the target storage doesn't exist
                storage.get_mut().swap_remove(entity_idx);
            } else {
                // else we move the component to the new destination
                target_storage_opt
                    .expect("Target is missing type_id storage")
                    .get_mut()
                    .take_from_other(storage.get_mut(), entity_idx);
            }
        }

        destination.entities.push(to_move);
    }

    pub(crate) fn add_to_entity_unchecked(&mut self, entity: EntityId, component: Dynamic) {
        let entity_idx = self
            .entities
            .iter()
            .position(|e| *e == entity)
            .expect("Could not find entity");

        let storage = self
            .components
            .get_mut(&component.inner_type())
            .expect("Could not find component storage");

        let storage_mut = storage.get_mut();

        assert_eq!(
            storage_mut.len(),
            entity_idx,
            "Not adding to the latest entry, will result in incoherent archetype"
        );

        component.add_to_vec(storage_mut);
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[track_caller]
    pub(crate) fn assert_coherent<const DEBUG_ONLY: bool>(&self) {
        if DEBUG_ONLY && !cfg!(debug_assertions) {
            return;
        }

        let mut encountered_entities: Vec<EntityId> = Vec::with_capacity(self.entities.len());

        for (i, entity_id) in self.entities.iter().enumerate() {
            let found_pos = encountered_entities
                .iter()
                .position(|encountered| encountered == entity_id);

            assert!(
                found_pos.is_none(),
                "Duplicate entity {:?}. First index {}, second {}",
                entity_id,
                found_pos.unwrap(),
                i
            );

            encountered_entities.push(*entity_id);
        }

        let expected_len = self.entities.len();

        for (component_type, component_vec) in self.components.iter() {
            unsafe {
                assert_eq!(
                    *component_type,
                    component_vec.get().as_ref().unwrap().inner_type_id(),
                    "AnyVec inner type does not match expected type"
                );

                assert_eq!(
                    expected_len,
                    component_vec.get().as_ref().unwrap().len(),
                    "Length mismatch for component vector for type {:?}",
                    component_type
                );
            }
        }
    }
}
