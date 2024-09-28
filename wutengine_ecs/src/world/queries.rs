use core::any::{Any, TypeId};
use core::cell::UnsafeCell;

use wutengine_core::{EntityId, ReadWriteDescriptor};
use wutengine_util_macro::make_combined_query_tuples;

use crate::archetype::{self, ArchetypeId};
use crate::vec::AnyVec;

use super::World;

impl World {
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
        let descriptors = C::get_query_descriptors();

        assert_unique_type_ids(&descriptors);

        let archetype_ids = self.archetype_ids_for(&descriptors);

        let type_ids: Vec<TypeId> = descriptors.iter().map(|desc| desc.type_id).collect();

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

    fn archetype_ids_for(&self, queried_components: &[QueryDescriptor]) -> Vec<ArchetypeId> {
        let required = queried_components
            .iter()
            .filter(|qc| matches!(qc.query_type, QueryType::Required));

        // Get only the archetype IDs for the required components.
        // If these archetypes happen to have one or more of the optional components,
        // then hey, we're in luck!
        self.archetype_ids_for_required(required)
    }

    fn archetype_ids_for_required<'a>(
        &self,
        queried_components: impl Iterator<Item = &'a QueryDescriptor>,
    ) -> Vec<ArchetypeId> {
        let mut archetype_set: Option<Vec<ArchetypeId>> = None;

        for queried_component in queried_components {
            let component_archetypes = match self.type_containers.get(&queried_component.type_id) {
                Some(archetype) => archetype,
                None => return Vec::new(),
            };

            if let Some(archetype_set) = &mut archetype_set {
                archetype_set.retain(|e| component_archetypes.contains(e));
            } else {
                archetype_set = Some(component_archetypes.clone());
            }

            if archetype_set.as_ref().unwrap().is_empty() {
                return Vec::new();
            }
        }

        archetype_set.unwrap_or_default()
    }
}

#[track_caller]
pub fn assert_unique_type_ids(ids: &[QueryDescriptor]) {
    let duplicate = get_first_duplicate_type_id(ids);

    if let Some(duplicate) = duplicate {
        panic!("Duplicate TypeId given: {:?}", duplicate)
    }
}

pub fn get_first_duplicate_type_id(ids: &[QueryDescriptor]) -> Option<TypeId> {
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            debug_assert_ne!(i, j);

            if ids[i].type_id == ids[j].type_id {
                return Some(ids[i].type_id);
            }
        }
    }

    None
}

pub trait Queryable<'q>: Sized {
    type Inner: Any;
    const READ_ONLY: bool;
    const QUERY_TYPE: QueryType;

    fn from_anyvec<'a: 'q>(num_entities: usize, cell: Option<&'a UnsafeCell<AnyVec>>) -> Vec<Self>;
}

impl<'q, T> Queryable<'q> for &'q T
where
    T: 'static,
{
    type Inner = T;
    const READ_ONLY: bool = true;
    const QUERY_TYPE: QueryType = QueryType::Required;

    fn from_anyvec<'a: 'q>(num_entities: usize, cell: Option<&'a UnsafeCell<AnyVec>>) -> Vec<Self> {
        let cell_ref = unsafe {
            cell.expect("None AnyVec given to required non-mutable queryable")
                .get()
                .as_ref::<'q>()
                .expect("UnsafeCell returned nullptr")
        };

        debug_assert_eq!(
            num_entities,
            cell_ref.len(),
            "Unexpected amount of components in AnyVec"
        );

        let mut output = Vec::with_capacity(cell_ref.len());

        for r in cell_ref.as_slice::<T>() {
            output.push(r);
        }

        output
    }
}

impl<'q, T> Queryable<'q> for &'q mut T
where
    T: 'static,
{
    type Inner = T;
    const READ_ONLY: bool = false;
    const QUERY_TYPE: QueryType = QueryType::Required;

    fn from_anyvec<'a: 'q>(num_entities: usize, cell: Option<&'a UnsafeCell<AnyVec>>) -> Vec<Self> {
        let cell_ref = unsafe {
            cell.expect("None AnyVec given to required mutable queryable")
                .get()
                .as_mut::<'q>()
                .expect("UnsafeCell returned nullptr")
        };

        debug_assert_eq!(
            num_entities,
            cell_ref.len(),
            "Unexpected amount of components in AnyVec"
        );

        let mut output = Vec::with_capacity(cell_ref.len());

        for r in cell_ref.as_mut_slice::<T>() {
            output.push(r);
        }

        output
    }
}

impl<'q, T> Queryable<'q> for Option<T>
where
    T: Queryable<'q>,
{
    type Inner = T::Inner;
    const READ_ONLY: bool = T::READ_ONLY;
    const QUERY_TYPE: QueryType = QueryType::Optional;

    fn from_anyvec<'a: 'q>(num_entities: usize, cell: Option<&'a UnsafeCell<AnyVec>>) -> Vec<Self> {
        match cell {
            Some(cell) => {
                let found_vec = T::from_anyvec(num_entities, Some(cell));

                found_vec.into_iter().map(Some).collect()
            }
            None => Vec::from_iter(std::iter::repeat_with(|| None).take(num_entities)),
        }
    }
}

pub trait CombinedQuery<'q>: Sized {
    fn get_query_descriptors() -> Vec<QueryDescriptor>;
    fn get_read_write_descriptors() -> Vec<ReadWriteDescriptor>;
    fn do_callback<F, O>(
        entities: &[EntityId],
        cells: Vec<Option<&'q UnsafeCell<AnyVec>>>,
        callback: F,
    ) -> Vec<O>
    where
        F: Fn(EntityId, Self) -> O;
}

#[derive(Debug, Clone, Copy)]
pub struct QueryDescriptor {
    pub type_id: TypeId,
    pub query_type: QueryType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Required,
    Optional,
    //TODO: NOT filters
    // Not,
}

impl<'q, T> CombinedQuery<'q> for T
where
    T: Queryable<'q>,
{
    fn get_query_descriptors() -> Vec<QueryDescriptor> {
        vec![QueryDescriptor {
            type_id: TypeId::of::<T::Inner>(),
            query_type: T::QUERY_TYPE,
        }]
    }

    fn get_read_write_descriptors() -> Vec<ReadWriteDescriptor> {
        vec![ReadWriteDescriptor {
            type_id: TypeId::of::<T::Inner>(),
            read_only: T::READ_ONLY,
        }]
    }

    fn do_callback<F, O>(
        entities: &[EntityId],
        cells: Vec<Option<&'q UnsafeCell<AnyVec>>>,
        callback: F,
    ) -> Vec<O>
    where
        F: Fn(EntityId, Self) -> O,
    {
        let refs = T::from_anyvec(entities.len(), cells[0]);
        let mut outputs = Vec::with_capacity(refs.len());

        for (args, &entity) in refs.into_iter().zip(entities) {
            outputs.push(callback(entity, args));
        }

        outputs
    }
}

macro_rules! make_all_combined_query_tuples {
    ($a:ident, $b: ident) => {
        make_combined_query_tuples!($a, $b);
    };

    ($a:ident, $b:ident, $($cs:ident),+) => {
        make_combined_query_tuples!($a, $b, $($cs),*);
        make_all_combined_query_tuples!($b, $($cs),*);
    };
}

make_all_combined_query_tuples!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
