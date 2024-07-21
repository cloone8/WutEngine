use core::{
    mem::{align_of, offset_of, size_of},
    ptr::{self, NonNull},
};
use std::alloc::Layout;

use nohash_hasher::IntMap;
use wutengine_core::{
    component::{Component, ComponentTypeId, DynComponent},
    entity::EntityId,
};

use crate::component::storage::ptr_helpers::debug_assert_aligned;

struct ComponentElement<T> {
    pub component: T,
    pub id: EntityId,
}

pub struct ComponentArray {
    component_id: ComponentTypeId,
    element_size: usize,
    element_align: usize,
    component_size: usize,
    component_align: usize,
    component_offset: usize,
    entity_id_offset: usize,
    drop_elem_fn: unsafe fn(*mut ()),
    entity_to_idx: IntMap<EntityId, usize>,
    len: usize,
    capacity: usize,
    alloc: Option<NonNull<u8>>,
}

impl ComponentArray {
    pub fn new_for<T: Component>() -> Self {
        Self {
            component_id: T::COMPONENT_ID,
            component_offset: offset_of!(ComponentElement<T>, component),
            entity_id_offset: offset_of!(ComponentElement<T>, id),
            element_size: size_of::<ComponentElement<T>>(),
            element_align: align_of::<ComponentElement<T>>(),
            component_size: size_of::<T>(),
            component_align: align_of::<T>(),
            drop_elem_fn: dynamic_drop::<ComponentElement<T>>,
            entity_to_idx: IntMap::default(),
            len: 0,
            capacity: 0,
            alloc: None,
        }
    }

    pub fn push(&mut self, entity: EntityId, component: Box<dyn DynComponent>) {
        debug_assert_eq!(self.component_id, component.get_dyn_component_id());

        self.ensure_capacity(self.len + 1);

        let as_raw = Box::into_raw(component) as *mut u8;

        let alloc = self.alloc.unwrap();

        unsafe {
            let (_, component_val_ptr, entity_id_ptr) = self.get_ptrs_for_element(alloc, self.len);

            std::ptr::copy_nonoverlapping(
                as_raw,
                component_val_ptr as *mut u8,
                self.component_size,
            );

            *(entity_id_ptr as *mut EntityId) = entity;
        }

        self.entity_to_idx.insert(entity, self.len);

        self.len += 1;
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    fn slice<T: Component>(&self) -> &[ComponentElement<T>] {
        debug_assert_eq!(self.component_id, T::COMPONENT_ID, "Component mismatch");

        if self.len == 0 {
            unsafe { std::slice::from_raw_parts(NonNull::dangling().as_ptr(), 0) }
        } else {
            let ptr = self
                .alloc
                .expect("Array with non-zero length should have an allocation");

            debug_assert!(
                (ptr.as_ptr() as *mut ComponentElement<T>).is_aligned(),
                "Alignment error!"
            );

            unsafe {
                std::slice::from_raw_parts(ptr.as_ptr() as *const ComponentElement<T>, self.len)
            }
        }
    }

    fn slice_mut<T: Component>(&mut self) -> &mut [ComponentElement<T>] {
        debug_assert_eq!(self.component_id, T::COMPONENT_ID, "Component mismatch");

        if self.len == 0 {
            unsafe { std::slice::from_raw_parts_mut(NonNull::dangling().as_ptr(), 0) }
        } else {
            let ptr = self
                .alloc
                .expect("Array with non-zero length should have an allocation");

            debug_assert!(
                (ptr.as_ptr() as *mut ComponentElement<T>).is_aligned(),
                "Alignment error!"
            );

            unsafe {
                std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut ComponentElement<T>, self.len)
            }
        }
    }

    pub fn get<T: Component>(&self, entity: EntityId) -> Option<&T> {
        debug_assert_eq!(self.component_id, T::COMPONENT_ID, "Component mismatch");
        self.entity_to_idx
            .get(&entity)
            .cloned()
            .map(|idx| &self.slice::<T>()[idx].component)
    }

    pub unsafe fn get_multi<T: Component>(&self, entities: &[EntityId]) -> Vec<Option<&T>> {
        debug_assert_eq!(self.component_id, T::COMPONENT_ID, "Component mismatch");
        debug_assert!(check_distinct(entities));

        let ids: Vec<Option<usize>> = entities
            .iter()
            .map(|id| self.entity_to_idx.get(&id).cloned())
            .collect();

        debug_assert!(
            ids.iter()
                .filter_map(|x| x.as_ref())
                .all(|id| *id < self.len),
            "Out of bounds!"
        );

        ids.into_iter()
            .map(|id| {
                id.and_then(|id| {
                    let (_, component, _) = self.get_ptrs_for_element(self.alloc.unwrap(), id);
                    (component as *const T).as_ref()
                })
            })
            .collect()
    }

    pub fn get_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        debug_assert_eq!(self.component_id, T::COMPONENT_ID, "Component mismatch");
        self.entity_to_idx
            .get(&entity)
            .cloned()
            .map(|idx| &mut self.slice_mut::<T>()[idx].component)
    }

    pub unsafe fn get_mut_multi<T: Component>(
        &mut self,
        entities: &[EntityId],
    ) -> Vec<Option<&mut T>> {
        let ids: Vec<Option<usize>> = entities
            .iter()
            .map(|id| self.entity_to_idx.get(&id).cloned())
            .collect();

        debug_assert!(
            ids.iter()
                .filter_map(|x| x.as_ref())
                .all(|id| *id < self.len),
            "Out of bounds!"
        );

        ids.into_iter()
            .map(|id| {
                id.and_then(|id| {
                    let (_, component, _) = self.get_ptrs_for_element(self.alloc.unwrap(), id);
                    (component as *mut T).as_mut()
                })
            })
            .collect()
    }

    #[inline]
    unsafe fn get_ptrs_for_element<T>(
        &self,
        mem: NonNull<T>,
        i: usize,
    ) -> (*mut (), *mut (), *mut ()) {
        let base = mem.as_ptr().byte_add(self.element_size * i);
        let component_val_ptr = base.byte_add(self.component_offset);
        let entity_id_ptr = base.byte_add(self.entity_id_offset);

        debug_assert_aligned!(base, self.element_align);
        debug_assert_aligned!(component_val_ptr, self.component_align);
        debug_assert_aligned!(entity_id_ptr, align_of::<EntityId>());

        (
            base as *mut (),
            component_val_ptr as *mut (),
            entity_id_ptr as *mut (),
        )
    }

    fn ensure_capacity(&mut self, required: usize) {
        if self.capacity >= required {
            return;
        }

        let alloc_ptr = unsafe {
            std::alloc::alloc(
                Layout::from_size_align(self.element_size * required, self.element_align).unwrap(),
            )
        };

        let new_alloc = NonNull::new(alloc_ptr).unwrap();

        if let Some(existing_alloc) = self.alloc.take() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    existing_alloc.as_ptr(),
                    new_alloc.as_ptr(),
                    self.element_size * self.len,
                );

                self.dealloc_current_alloc();
            }
        }

        self.alloc = Some(new_alloc);
        self.capacity = required;
    }

    unsafe fn dealloc_current_alloc(&mut self) {
        if let Some(alloc) = self.alloc.take() {
            unsafe {
                for i in 0..self.len {
                    let (base, _, _) = self.get_ptrs_for_element(alloc, i);

                    (self.drop_elem_fn)(base)
                }

                let layout =
                    Layout::from_size_align(self.element_size * self.capacity, self.element_align)
                        .unwrap();

                std::alloc::dealloc(alloc.as_ptr(), layout);
            }

            self.capacity = 0;
        }
    }
}

fn check_distinct<T: PartialEq>(arr: &[T]) -> bool {
    let mut found: Vec<&T> = Vec::with_capacity(arr.len());

    for elem in arr {
        if found.contains(&elem) {
            return false;
        }

        found.push(elem);
    }

    true
}

unsafe fn dynamic_drop<T>(raw: *mut ()) {
    ptr::drop_in_place::<T>(raw as *mut T);
}

impl Drop for ComponentArray {
    fn drop(&mut self) {
        unsafe {
            self.dealloc_current_alloc();
        }
    }
}
