//! OpenGL arrays with std140 layout

use core::ops::{Deref, DerefMut};

/// An OpenGL array element, forced to std140 layout
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArrayElem<T>(pub(crate) T);

impl<T> From<T> for ArrayElem<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for ArrayElem<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ArrayElem<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// An OpenGL array with std140 layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlArray<T, const N: usize>(pub(crate) [ArrayElem<T>; N]);

impl<T, const N: usize> From<[T; N]> for GlArray<T, N>
where
    T: Clone,
{
    fn from(value: [T; N]) -> Self {
        Self(std::array::from_fn(|i| ArrayElem(value[i].clone())))
    }
}
