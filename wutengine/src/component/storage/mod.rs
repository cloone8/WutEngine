use array::ComponentArray;
use wutengine_core::{
    component::{Component, DynComponent},
    entity::EntityId,
};

mod array;

#[macro_use]
mod ptr_helpers;

pub struct ComponentElement<T> {
    pub component: T,
    pub id: EntityId,
}

pub enum ComponentStorage {
    Array(ComponentArray),
}

impl ComponentStorage {
    pub fn new_for<T: Component>(kind: StorageKind) -> Self {
        match kind {
            StorageKind::Array => Self::Array(ComponentArray::new_for::<T>()),
        }
    }

    pub fn push(&mut self, entity: EntityId, component: Box<dyn DynComponent>) {
        match self {
            ComponentStorage::Array(storage) => storage.push(entity, component),
        }
    }

    pub const fn len(&self) -> usize {
        match self {
            ComponentStorage::Array(storage) => storage.len(),
        }
    }

    pub fn get<T: Component>(&self, entity: EntityId) -> Option<&T> {
        match self {
            ComponentStorage::Array(storage) => storage.get(entity),
        }
    }

    pub unsafe fn get_multi<T: Component>(&self, entities: &[EntityId]) -> Vec<Option<&T>> {
        match self {
            ComponentStorage::Array(storage) => storage.get_multi(entities),
        }
    }

    pub fn get_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        match self {
            ComponentStorage::Array(storage) => storage.get_mut(entity),
        }
    }

    pub unsafe fn get_mut_multi<T: Component>(
        &mut self,
        entities: &[EntityId],
    ) -> Vec<Option<&mut T>> {
        match self {
            ComponentStorage::Array(storage) => storage.get_mut_multi(entities),
        }
    }

    pub fn all<T: Component>(&self) -> &[ComponentElement<T>] {
        match self {
            ComponentStorage::Array(storage) => storage.slice(),
        }
    }

    pub fn all_mut<T: Component>(&mut self) -> &mut [ComponentElement<T>] {
        match self {
            ComponentStorage::Array(storage) => storage.slice_mut(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StorageKind {
    Array,
}
