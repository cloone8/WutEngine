use core::any::{Any, TypeId};
use core::cell::UnsafeCell;

use wutengine_core::{EntityId, ReadWriteDescriptor};
use wutengine_util_macro::make_combined_query_tuples;

use crate::archetype::ArchetypeId;
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

    fn archetype_ids_for(&self, type_ids: &[TypeId]) -> Vec<ArchetypeId> {
        let mut init_archetypes = match self.type_containers.get(&type_ids[0]) {
            Some(archetypes) => archetypes.clone(),
            None => return Vec::new(),
        };

        for type_id in &type_ids[1..] {
            let containing_archetypes = match self.type_containers.get(type_id) {
                Some(archetypes) => archetypes,
                None => return Vec::new(),
            };

            init_archetypes.retain(|e| containing_archetypes.contains(e));

            if init_archetypes.is_empty() {
                // Short-circuit
                return Vec::new();
            }
        }

        init_archetypes
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

pub trait Queryable<'q>: Sized {
    type Inner: Any;
    const READ_ONLY: bool;

    fn from_anyvec<'a: 'q>(cell: &'a UnsafeCell<AnyVec>) -> Vec<Self>;
}

impl<'q, T> Queryable<'q> for &'q T
where
    T: 'static,
{
    type Inner = T;
    const READ_ONLY: bool = true;

    fn from_anyvec<'a: 'q>(cell: &'a UnsafeCell<AnyVec>) -> Vec<Self> {
        let cell_ref = unsafe {
            cell.get()
                .as_ref::<'q>()
                .expect("UnsafeCell returned nullptr")
        };

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

    fn from_anyvec<'a: 'q>(cell: &'a UnsafeCell<AnyVec>) -> Vec<Self> {
        let cell_ref = unsafe {
            cell.get()
                .as_mut::<'q>()
                .expect("UnsafeCell returned nullptr")
        };

        let mut output = Vec::with_capacity(cell_ref.len());

        for r in cell_ref.as_mut_slice::<T>() {
            output.push(r);
        }

        output
    }
}

pub trait CombinedQuery<'q>: Sized {
    fn get_type_ids() -> Vec<TypeId>;
    fn get_descriptors() -> Vec<ReadWriteDescriptor>;
    fn do_callback<F, O>(
        entities: &[EntityId],
        cells: Vec<&'q UnsafeCell<AnyVec>>,
        callback: F,
    ) -> Vec<O>
    where
        F: Fn(EntityId, Self) -> O;
}

impl<'q, T> CombinedQuery<'q> for T
where
    T: Queryable<'q>,
{
    fn get_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T::Inner>()]
    }

    fn get_descriptors() -> Vec<ReadWriteDescriptor> {
        vec![ReadWriteDescriptor {
            type_id: TypeId::of::<T::Inner>(),
            read_only: T::READ_ONLY,
        }]
    }

    fn do_callback<F, O>(
        entites: &[EntityId],
        cells: Vec<&'q UnsafeCell<AnyVec>>,
        callback: F,
    ) -> Vec<O>
    where
        F: Fn(EntityId, Self) -> O,
    {
        let refs = T::from_anyvec(cells[0]);
        let mut outputs = Vec::with_capacity(refs.len());

        for (args, &entity) in refs.into_iter().zip(entites) {
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
