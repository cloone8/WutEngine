use core::{any::Any, hash::BuildHasherDefault};
use std::collections::HashMap;

use nohash_hasher::{BuildNoHashHasher, IntMap, NoHashHasher};
use wutengine_core::{Component, EntityId};

#[derive(Debug)]
pub struct ComponentSet {
    inner: Box<dyn Any>,
}

impl ComponentSet {
    pub fn new<T: Component>() -> Self {
        let actual_val: IntMap<EntityId, T> = IntMap::default();

        let boxed: Box<dyn Any> = Box::new(actual_val);

        Self { inner: boxed }
    }

    pub fn get_as<T: Component>(&self) -> Option<&IntMap<EntityId, T>> {
        self.inner.as_ref().downcast_ref()
    }

    pub fn get_mut_as<T: Component>(&mut self) -> Option<&mut IntMap<EntityId, T>> {
        self.inner.as_mut().downcast_mut()
    }
}

#[cfg(test)]
mod tests {
    use wutengine_core::Component;

    use super::ComponentSet;

    struct DummyComponent;

    impl Component for DummyComponent {}

    #[test]
    fn test_downcast() {
        let mut testset = ComponentSet::new::<DummyComponent>();

        assert!(testset.get_as::<DummyComponent>().is_some());
        assert!(testset.get_mut_as::<DummyComponent>().is_some());
    }
}
