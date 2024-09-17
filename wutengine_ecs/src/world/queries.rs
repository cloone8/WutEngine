use core::any::Any;
use core::cell::UnsafeCell;

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
