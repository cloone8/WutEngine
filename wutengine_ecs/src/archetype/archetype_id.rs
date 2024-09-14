use core::any::TypeId;
use core::hash::{Hash, Hasher};
use std::hash::DefaultHasher;

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArchetypeId(u64);

impl ArchetypeId {
    pub fn new(type_ids: &[TypeId]) -> Self {
        let mut ids_sorted = Vec::from(type_ids);
        ids_sorted.sort();

        let mut hasher = DefaultHasher::default();

        ids_sorted.hash(&mut hasher);

        Self(hasher.finish())
    }
}
