use core::mem::MaybeUninit;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArrayElem<T>(pub(crate) T);

impl<T> From<T> for ArrayElem<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        Self(value)
    }
}

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
