use core::any::{Any, TypeId};

use crate::vec::AnyVecStorageDescriptor;

#[derive(Debug, Clone)]
pub(crate) struct TypeDescriptorSet {
    pub descriptors: Vec<(TypeId, AnyVecStorageDescriptor)>,
}

impl TypeDescriptorSet {
    pub(crate) fn new_empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }
    pub(crate) fn new<T: Any>() -> Self {
        Self {
            descriptors: vec![(TypeId::of::<T>(), AnyVecStorageDescriptor::new::<T>())],
        }
    }

    pub(crate) fn add<T: Any>(&mut self) {
        self.descriptors
            .push((TypeId::of::<T>(), AnyVecStorageDescriptor::new::<T>()));
    }
}
