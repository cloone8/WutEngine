use core::any::{Any, TypeId};
use core::cell::UnsafeCell;

use itertools::izip;

use crate::vec::AnyVec;

pub trait Queryable<'q>: Sized {
    type Inner: Any;

    fn from_anyvec<'a: 'q>(cell: &'a UnsafeCell<AnyVec>) -> Vec<Self>;
}

impl<'q, T> Queryable<'q> for &'q T
where
    T: 'static,
{
    type Inner = T;

    fn from_anyvec<'a: 'q>(cell: &'a UnsafeCell<AnyVec>) -> Vec<Self> {
        let mut output = Vec::new();

        let cell_ref = unsafe {
            cell.get()
                .as_ref::<'q>()
                .expect("UnsafeCell returned nullptr")
        };

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

    fn from_anyvec<'a: 'q>(cell: &'a UnsafeCell<AnyVec>) -> Vec<Self> {
        let mut output = Vec::new();

        let cell_ref = unsafe {
            cell.get()
                .as_mut::<'q>()
                .expect("UnsafeCell returned nullptr")
        };

        for r in cell_ref.as_mut_slice::<T>() {
            output.push(r);
        }

        output
    }
}

pub trait CombinedQuery<'q>: Sized {
    fn get_type_ids() -> Vec<TypeId>;
    fn do_callback(cells: Vec<&'q UnsafeCell<AnyVec>>, callback: impl FnMut(Self));
}

impl<'q, T> CombinedQuery<'q> for T
where
    T: Queryable<'q>,
{
    fn get_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T::Inner>()]
    }

    fn do_callback(cells: Vec<&'q UnsafeCell<AnyVec>>, mut callback: impl FnMut(Self)) {
        let refs = T::from_anyvec(cells[0]);

        for args in refs {
            callback(args);
        }
    }
}

impl<'q, A, B> CombinedQuery<'q> for (A, B)
where
    A: Queryable<'q>,
    B: Queryable<'q>,
{
    fn get_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A::Inner>(), TypeId::of::<B::Inner>()]
    }

    fn do_callback(cells: Vec<&'q UnsafeCell<AnyVec>>, mut callback: impl FnMut(Self)) {
        assert_eq!(2, cells.len());

        let refs_a = A::from_anyvec(cells[0]);
        let refs_b = B::from_anyvec(cells[1]);

        debug_assert_eq!(refs_a.len(), refs_b.len());

        let combined = izip!(refs_a, refs_b);

        for args in combined {
            callback(args);
        }
    }
}

impl<'q, A, B, C> CombinedQuery<'q> for (A, B, C)
where
    A: Queryable<'q>,
    B: Queryable<'q>,
    C: Queryable<'q>,
{
    fn get_type_ids() -> Vec<TypeId> {
        vec![
            TypeId::of::<A::Inner>(),
            TypeId::of::<B::Inner>(),
            TypeId::of::<C::Inner>(),
        ]
    }

    fn do_callback(cells: Vec<&'q UnsafeCell<AnyVec>>, mut callback: impl FnMut(Self)) {
        assert_eq!(3, cells.len());

        let refs_a = A::from_anyvec(cells[0]);
        let refs_b = B::from_anyvec(cells[1]);
        let refs_c = C::from_anyvec(cells[2]);

        debug_assert_eq!(refs_a.len(), refs_b.len());
        debug_assert_eq!(refs_a.len(), refs_c.len());

        let combined = izip!(refs_a, refs_b, refs_c);

        for args in combined {
            callback(args);
        }
    }
}

// let refs_a = A::from_anyvec(components[0]);
// let refs_b = B::from_anyvec(components[1]);
// let refs_c = C::from_anyvec(components[2]);

// debug_assert_eq!(refs_a.len(), refs_b.len());
// debug_assert_eq!(refs_a.len(), refs_c.len());

// let combined = izip!(refs_a, refs_b, refs_c);

// for x in combined {
//     callback(x.0, x.1, x.2);
// }
