use std::cell::UnsafeCell;

use nohash_hasher::IntMap;
use wutengine_core::{Component, ComponentTypeId, EntityId};
use wutengine_util_macro::make_queryable_tuples;

use crate::legacy_storage::ComponentStorage;

pub unsafe trait Queryable<'a>: Sized {
    fn reads() -> Vec<ComponentTypeId>;
    fn writes() -> Vec<ComponentTypeId>;

    fn do_query(
        entities: &'a [EntityId],
        components: &'a IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
    ) -> Vec<(EntityId, Option<Self>)>;
}

unsafe impl<'a, T> Queryable<'a> for &'a T
where
    T: Component,
{
    fn do_query(
        entities: &'a [EntityId],
        components: &'a IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
    ) -> Vec<(EntityId, Option<Self>)> {
        let storage_cell = components.get(&T::COMPONENT_ID).unwrap();
        let storage = unsafe { storage_cell.get().as_ref::<'a>().unwrap() };

        let components = unsafe { storage.get_multi::<T>(entities) };

        entities.iter().copied().zip(components).collect()
    }

    fn reads() -> Vec<ComponentTypeId> {
        vec![T::COMPONENT_ID]
    }

    fn writes() -> Vec<ComponentTypeId> {
        Vec::new()
    }
}

unsafe impl<'a, T> Queryable<'a> for &'a mut T
where
    T: Component,
{
    fn do_query(
        entities: &'a [EntityId],
        components: &'a IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
    ) -> Vec<(EntityId, Option<Self>)> {
        let storage_cell = components.get(&T::COMPONENT_ID).unwrap();
        let storage = unsafe { storage_cell.get().as_mut::<'a>().unwrap() };

        let components = unsafe { storage.get_mut_multi::<T>(entities) };

        entities.iter().copied().zip(components).collect()
    }

    fn reads() -> Vec<ComponentTypeId> {
        Vec::new()
    }

    fn writes() -> Vec<ComponentTypeId> {
        vec![T::COMPONENT_ID]
    }
}

unsafe impl<'a, T> Queryable<'a> for Option<T>
where
    T: Queryable<'a>,
{
    fn reads() -> Vec<ComponentTypeId> {
        T::reads()
    }

    fn writes() -> Vec<ComponentTypeId> {
        T::writes()
    }

    fn do_query(
        entities: &'a [EntityId],
        components: &'a IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
    ) -> Vec<(EntityId, Option<Self>)> {
        let inner_result = T::do_query(entities, components);

        inner_result
            .into_iter()
            .map(|(id, comp)| (id, Some(comp)))
            .collect()
    }
}

pub struct World<'a> {
    entities: &'a [EntityId],
    components: &'a IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
}

impl<'a> World<'a> {
    pub(crate) unsafe fn new(
        entities: &'a [EntityId],
        components: &'a IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
    ) -> Self {
        Self {
            entities,
            components,
        }
    }

    pub fn query<T: Queryable<'a>>(&'a self) -> Vec<(EntityId, T)> {
        T::do_query(self.entities, self.components)
            .into_iter()
            .filter(|(_, comp)| comp.is_some())
            .map(|(id, comp)| (id, comp.unwrap()))
            .collect()
    }
}

macro_rules! make_all_queryable_tuples {
    ($a:ident, $b: ident) => {
        make_queryable_tuples!($a, $b);
    };

    ($a:ident, $b:ident, $($cs:ident),+) => {
        make_queryable_tuples!($a, $b, $($cs),*);
        make_all_queryable_tuples!($b, $($cs),*);
    };
}

make_all_queryable_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
