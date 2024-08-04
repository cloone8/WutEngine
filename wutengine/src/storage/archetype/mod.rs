use std::any::{Any, TypeId};
use std::cmp::Ordering;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Archetype {
    inner: Box<[TypeId]>,
}

impl FromIterator<TypeId> for Archetype {
    fn from_iter<T: IntoIterator<Item = TypeId>>(iter: T) -> Self {
        let mut boxed = Vec::from_iter(iter).into_boxed_slice();

        assert_ne!(0, boxed.len());

        boxed.sort();

        if cfg!(debug_assertions) {
            for i in 1..boxed.len() {
                debug_assert_ne!(boxed[i - 1], boxed[i]);
            }
        }

        Self { inner: boxed }
    }
}

impl<'a> FromIterator<&'a TypeId> for Archetype {
    fn from_iter<T: IntoIterator<Item = &'a TypeId>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().copied())
    }
}

impl Archetype {
    pub fn new(types: &[TypeId]) -> Self {
        Self::from_iter(types.iter().copied())
    }

    #[must_use]
    pub fn add_type<T: Any>(&self) -> Archetype {
        let orig_iter = self.inner.iter().copied();
        let added_iter = std::iter::once(TypeId::of::<T>());

        Archetype::from_iter(orig_iter.chain(added_iter))
    }

    #[must_use]
    pub fn add_value(&self, val: TypeId) -> Archetype {
        self.add_values(&[val])
    }

    #[must_use]
    pub fn add_values(&self, types: &[TypeId]) -> Archetype {
        let orig_iter = self.inner.iter();

        Archetype::from_iter(orig_iter.chain(types.iter()))
    }

    #[inline]
    pub fn index_of<T: Any>(&self) -> Option<usize> {
        self.index_of_val(TypeId::of::<T>())
    }

    #[inline]
    pub fn index_of_val(&self, typeid: TypeId) -> Option<usize> {
        self.inner.binary_search(&typeid).ok()
    }

    pub fn is_subset_of(&self, other: &Archetype) -> bool {
        let mut other_idx: usize = 0;
        let mut self_idx: usize = 0;

        while other_idx < other.inner.len() && self_idx < self.inner.len() {
            let cmp_result = other.inner[other_idx].cmp(&self.inner[self_idx]);

            match cmp_result {
                Ordering::Less => other_idx += 1,
                Ordering::Equal => self_idx += 1,
                Ordering::Greater => return false,
            }
        }

        self_idx == self.inner.len()
    }
}
