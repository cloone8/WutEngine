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
use wutengine_util_macro::generate_component_filter_for_tuple;

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

// TODO: All these impls in a macro

impl<'a, A, B> ComponentFilter<'a> for (&'a A, &'a B)
where
    A: Component,
    B: Component,
{
    type Output<'o> = (&'o A, &'o B);

    fn filter<F>(
        entities: &[EntityId],
        components: &'a IntMap<ComponentTypeId, RefCell<ComponentStorage>>,
        func: F,
    ) where
        F: for<'x> FnOnce(Vec<(EntityId, Self::Output<'x>)>),
    {
        let a_storage = components.get(&A::COMPONENT_ID).unwrap();
        let b_storage = components.get(&B::COMPONENT_ID).unwrap();

        let a = a_storage.borrow();
        let b = b_storage.borrow();

        let a_components = unsafe { a.get_multi::<A>(entities) };
        let b_components = unsafe { b.get_multi::<B>(entities) };

        let result = entities
            .iter()
            .copied()
            .zip(itertools::izip!(a_components, b_components))
            .filter(|(_, components)| components.0.is_some())
            .filter(|(_, components)| components.1.is_some())
            .map(|(id, components)| (id, (components.0.unwrap(), components.1.unwrap())))
            .collect();

        func(result);
    }
}

impl<'a, A, B> ComponentFilter<'a> for (&'a mut A, &'a B)
where
    A: Component,
    B: Component,
{
    type Output<'o> = (&'o mut A, &'o B);

    fn filter<F>(
        entities: &[EntityId],
        components: &'a IntMap<ComponentTypeId, RefCell<ComponentStorage>>,
        func: F,
    ) where
        F: for<'x> FnOnce(Vec<(EntityId, Self::Output<'x>)>),
    {
        let a_storage = components.get(&A::COMPONENT_ID).unwrap();
        let b_storage = components.get(&B::COMPONENT_ID).unwrap();

        let mut a = a_storage.borrow_mut();
        let b = b_storage.borrow();

        let a_components = unsafe { a.get_mut_multi::<A>(entities) };
        let b_components = unsafe { b.get_multi::<B>(entities) };

        let result = entities
            .iter()
            .copied()
            .zip(itertools::izip!(a_components, b_components))
            .filter(|(_, components)| components.0.is_some())
            .filter(|(_, components)| components.1.is_some())
            .map(|(id, components)| (id, (components.0.unwrap(), components.1.unwrap())))
            .collect();

        func(result);
    }
}

impl<'a, A, B> ComponentFilter<'a> for (Option<&'a A>, &'a B)
where
    A: Component,
    B: Component,
{
    type Output<'o> = (Option<&'o A>, &'o B);

    fn filter<F>(
        entities: &[EntityId],
        components: &'a IntMap<ComponentTypeId, RefCell<ComponentStorage>>,
        func: F,
    ) where
        F: for<'x> FnOnce(Vec<(EntityId, Self::Output<'x>)>),
    {
        let a_storage = components.get(&A::COMPONENT_ID).unwrap();
        let b_storage = components.get(&B::COMPONENT_ID).unwrap();

        let a = a_storage.borrow();
        let b = b_storage.borrow();

        let a_components = unsafe { a.get_multi::<A>(entities) };
        let b_components = unsafe { b.get_multi::<B>(entities) };

        let result = entities
            .iter()
            .copied()
            .zip(itertools::izip!(a_components, b_components))
            .filter(|(_, components)| components.1.is_some())
            .map(|(id, components)| (id, (components.0, components.1.unwrap())))
            .collect();

        func(result);
    }
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
