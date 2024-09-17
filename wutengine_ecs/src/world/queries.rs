use core::any::{Any, TypeId};
use core::cell::UnsafeCell;

use wutengine_util_macro::make_combined_query_tuples;

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
