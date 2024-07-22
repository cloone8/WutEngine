use core::{
    cell::RefCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use std::sync::{RwLock, RwLockReadGuard};

use nohash_hasher::IntMap;
use wutengine_core::{
    component::{self, Component, ComponentTypeId},
    entity::EntityId,
};

use crate::component::storage::ComponentStorage;

pub trait ComponentFilter<'a>: Sized {
    type Output<'o>;

    fn filter<F>(
        entities: &[EntityId],
        components: &'a IntMap<ComponentTypeId, RefCell<ComponentStorage>>,
        func: F,
    ) where
        F: for<'x> FnOnce(Vec<(EntityId, Self::Output<'x>)>);
}

pub struct World<'a> {
    entities: &'a [EntityId],
    components: &'a IntMap<ComponentTypeId, RefCell<ComponentStorage>>,
}

impl<'a> World<'a> {
    pub(crate) fn new(
        entities: &'a [EntityId],
        components: &'a IntMap<ComponentTypeId, RefCell<ComponentStorage>>,
    ) -> Self {
        Self {
            entities,
            components,
        }
    }

    pub fn query<T: Component>(&'a self, func: impl FnOnce(Vec<(EntityId, &T)>)) {
        let err_str = "Unknown component type!";

        let a_arr = self
            .components
            .get(&T::COMPONENT_ID)
            .expect(err_str)
            .borrow();

        let params = unsafe {
            let a_entities = a_arr.get_multi::<T>(self.entities);

            self.entities
                .iter()
                .cloned()
                .zip(a_entities)
                .filter(|(_, component)| component.is_some())
                .map(|(entity, component)| (entity, component.unwrap()))
                .collect()
        };

        func(params);
    }

    pub fn query_mut<T: Component>(&'a self, func: impl FnOnce(Vec<(EntityId, &mut T)>)) {
        let unknown_component_err_str = "Unknown component type!";

        let mut a_arr = self
            .components
            .get(&T::COMPONENT_ID)
            .expect(unknown_component_err_str)
            .borrow_mut();

        let params = unsafe {
            let a_entities = a_arr.get_mut_multi::<T>(self.entities);

            self.entities
                .iter()
                .cloned()
                .zip(a_entities)
                .filter(|(_, component)| component.is_some())
                .map(|(entity, component)| (entity, component.unwrap()))
                .collect()
        };

        func(params);
    }

    pub fn query_multiple<F: ComponentFilter<'a>>(
        &'a self,
        func: impl for<'x> FnOnce(Vec<(EntityId, F::Output<'x>)>),
    ) {
        F::filter(self.entities, self.components, func);
    }
}

wutengine_util_macro::make_componentfilter_tuples!(5);
