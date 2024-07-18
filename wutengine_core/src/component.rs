use core::{any::Any, fmt::Debug};
use std::hash::{Hash, Hasher};

use nohash_hasher::IsEnabled;
use static_assertions::assert_obj_safe;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentTypeId(u64);

impl Hash for ComponentTypeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl IsEnabled for ComponentTypeId {}

impl ComponentTypeId {
    pub const fn from_int(val: u64) -> Self {
        Self(val)
    }
}

pub trait DynComponent: Any + Debug {
    fn get_dyn_component_id(&self) -> ComponentTypeId;
}

pub trait Component: DynComponent + Sized {
    const COMPONENT_ID: ComponentTypeId;
}

assert_obj_safe!(DynComponent);
