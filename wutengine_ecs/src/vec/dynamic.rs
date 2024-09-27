use core::any::{Any, TypeId};
use core::fmt::Debug;

use crate::archetype::TypeDescriptorSet;

use super::AnyVec;

type ModTypeDescriptorFunc =
    dyn for<'a> Fn(Option<&'a mut TypeDescriptorSet>) -> Option<TypeDescriptorSet>;

type AddToAnyVecFunc = dyn for<'a> FnOnce(Option<&'a mut AnyVec>) -> Option<AnyVec>;

pub struct Dynamic {
    inner_type: TypeId,
    type_descriptor_fn: Box<ModTypeDescriptorFunc>,
    add_fn: Box<AddToAnyVecFunc>,
}

impl Debug for Dynamic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dynamic")
            .field("inner_type", &self.inner_type)
            .finish()
    }
}

impl Dynamic {
    pub fn new<T: Any>(val: T) -> Self {
        Self {
            inner_type: TypeId::of::<T>(),
            add_fn: Box::new(|anyvec| match anyvec {
                Some(anyvec) => {
                    anyvec.push::<T>(val);
                    None
                }
                None => {
                    let mut anyvec = AnyVec::new::<T>();
                    anyvec.push::<T>(val);
                    Some(anyvec)
                }
            }),
            type_descriptor_fn: Box::new(|type_descriptor| match type_descriptor {
                Some(td) => {
                    td.add::<T>();
                    None
                }
                None => Some(TypeDescriptorSet::new::<T>()),
            }),
        }
    }

    #[inline]
    pub(crate) const fn inner_type(&self) -> TypeId {
        self.inner_type
    }

    #[inline]
    pub(crate) fn add_to_vec(self, vec: &mut AnyVec) {
        let ret = (self.add_fn)(Some(vec));

        debug_assert!(ret.is_none(), "Unexpected anyvec returned");
    }

    #[inline]
    #[must_use]
    pub(crate) fn add_to_new_vec(self) -> AnyVec {
        (self.add_fn)(None).expect("No AnyVec returned!")
    }

    #[inline]
    #[expect(
        dead_code,
        reason = "Will be used later when I de-crappify the ECS multi-component-add code"
    )]
    pub(crate) fn add_type_to_descriptor(&self, tds: &mut TypeDescriptorSet) {
        let ret = (self.type_descriptor_fn)(Some(tds));

        debug_assert!(ret.is_none(), "Unexpected typedescriptorset returned");
    }

    #[inline]
    #[must_use]
    pub(crate) fn add_type_to_new_descriptor(&self) -> TypeDescriptorSet {
        (self.type_descriptor_fn)(None).expect("No TypeDescriptorSet returned!")
    }
}
