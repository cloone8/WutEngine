use nohash_hasher::IntMap;
use wutengine_core::{
    component::{Component, ComponentTypeId},
    entity::EntityId,
};
use wutengine_util_macro::generate_component_filter_for_tuple;

use crate::component::storage::array::ComponentArray;

pub trait ComponentFilter {
    type Output<'a>;
    type OutputMut<'a>;

    fn filter(components: &IntMap<ComponentTypeId, ComponentArray>) -> Self::Output<'_>;
    fn filter_mut(components: &mut IntMap<ComponentTypeId, ComponentArray>) -> Self::OutputMut<'_>;
}

pub struct World<'a> {
    components: &'a mut IntMap<ComponentTypeId, ComponentArray>,
}

impl<'a> World<'a> {
    pub(crate) fn new(components: &'a mut IntMap<ComponentTypeId, ComponentArray>) -> Self {
        Self { components }
    }

    pub fn query<T: Component>(&'a self) -> Vec<&'a T> {
        let err_str = "Unknown component type!";

        let a_arr = self.components.get(&T::COMPONENT_ID).expect(err_str);

        let entity_ids: Vec<EntityId> = a_arr.slice::<T>().iter().map(|x| x.id).collect();

        unsafe {
            let a_entities = a_arr.get_multi::<T>(&entity_ids);

            a_entities.into_iter().flatten().collect()
        }
    }

    pub fn query_mut<T: Component>(&'a mut self) -> Vec<&'a mut T> {
        let unknown_component_err_str = "Unknown component type!";

        let a_arr = self
            .components
            .get_mut(&T::COMPONENT_ID)
            .expect(unknown_component_err_str);

        let entity_ids: Vec<EntityId> = a_arr.slice::<T>().iter().map(|x| x.id).collect();

        unsafe {
            let a_entities = a_arr.get_mut_multi::<T>(&entity_ids);

            a_entities.into_iter().flatten().collect()
        }
    }

    pub fn query_multiple<F: ComponentFilter>(&'a self) -> F::Output<'a> {
        F::filter(self.components)
    }

    pub fn query_multiple_mut<F: ComponentFilter>(&'a mut self) -> F::OutputMut<'a> {
        F::filter_mut(self.components)
    }
}

macro_rules! generate_all_component_filters_tuples {
    ($a:ident, $b: ident) => {
        generate_component_filter_for_tuple!($a, $b);
    };

    ($a:ident, $b:ident, $($cs:ident),+) => {
        generate_component_filter_for_tuple!($a, $b, $($cs),*);
        generate_all_component_filters_tuples!($b, $($cs),*);
    };
}

generate_all_component_filters_tuples!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
