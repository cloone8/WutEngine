use core::any::{Any, TypeId};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use std::collections::HashMap;

use crate::entity_id::EntityId;
use crate::vec::AnyVec;

mod archetype_id;

pub use archetype_id::*;

#[derive(Debug)]
pub struct Archetype {
    entities: Vec<EntityId>,
    components: HashMap<TypeId, UnsafeCell<AnyVec>>,
}

#[derive(Debug)]
pub struct ArchetypeMapMut<'a> {
    inner: &'a mut Archetype,
}

impl<'a> ArchetypeMapMut<'a> {
    pub fn new(archetype: &'a mut Archetype) -> Self {
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

    pub fn new_single<T: Any>(entity: EntityId, value: T) -> Self {
        let mut component_vec = AnyVec::new::<T>();
        component_vec.push(value);

        let mut components_map = HashMap::default();
        components_map.insert(TypeId::of::<T>(), UnsafeCell::new(component_vec));

        Archetype {
            entities: vec![entity],
            components: components_map,
        }
    }

    pub fn get_contained_entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn get_contained_types(&self) -> impl Iterator<Item = &TypeId> {
        self.components.keys()
    }

    pub fn get_components_for_read(&self, types: &[TypeId]) -> Vec<&UnsafeCell<AnyVec>> {
        types
            .iter()
            .map(|t| self.components.get(t).expect("Unknown TypeId"))
            .collect()
    }

    pub fn get_components_for_add(&mut self, to_add: EntityId) -> ArchetypeMapMut {
        debug_assert!(
            !self.entities.contains(&to_add),
            "Entity already present in archetype: {:?}",
            to_add
        );

        self.entities.push(to_add);

        self.mutmap()
    }

    pub fn get_components_for_remove(&mut self, to_remove: EntityId) -> (usize, ArchetypeMapMut) {
        let entity_idx = self
            .entities
            .iter()
            .position(|e| *e == to_remove)
            .unwrap_or_else(|| panic!("Entity not present in archetype: {:?}", to_remove));

        self.entities.swap_remove(entity_idx);

        (entity_idx, self.mutmap())
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[track_caller]
    pub fn assert_coherent<const DEBUG_ONLY: bool>(&self) {
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
