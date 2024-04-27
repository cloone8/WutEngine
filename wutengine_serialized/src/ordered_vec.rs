use std::ops::{Deref, DerefMut};

pub struct OrderedVec<T> {
    inner: Vec<T>
}

impl<T> Deref for OrderedVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for OrderedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
