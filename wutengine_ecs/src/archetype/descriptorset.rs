use core::any::{Any, TypeId};

use crate::vec::AnyVecStorageDescriptor;

#[derive(Debug, Clone)]
pub(crate) struct TypeDescriptorSet {
    pub descriptors: Vec<(TypeId, AnyVecStorageDescriptor)>,
}

impl TypeDescriptorSet {
    pub fn new_empty() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }
    pub fn new<T: Any>() -> Self {
        Self {
            descriptors: vec![(TypeId::of::<T>(), AnyVecStorageDescriptor::new::<T>())],
        }
    }

    pub fn add<T: Any>(&mut self) {
        self.descriptors
            .push((TypeId::of::<T>(), AnyVecStorageDescriptor::new::<T>()));
    }
}
