use core::{
    mem::{offset_of, size_of},
    ptr::{self, NonNull},
};
use std::alloc::Layout;

use nohash_hasher::IntMap;
use wutengine_core::{
    component::{Component, ComponentTypeId, DynComponent},
    entity::EntityId,
};

#[repr(C)]
struct LayoutHelper<T> {
    id: EntityId,
    val: T,
}

pub struct ComponentArray {
    component_id: ComponentTypeId,
    unpadded_component_size: usize,
    elem_layout: Layout,
    drop_elem_fn: unsafe fn(*mut ()),
    entity_to_idx: IntMap<EntityId, usize>,
    len: usize,
    capacity: usize,
    alloc: Option<NonNull<u8>>,
}

impl ComponentArray {
    pub fn new_for<T: Component + Sized>() -> Self {
        debug_assert_eq!(
            size_of::<EntityId>(),
            offset_of!(LayoutHelper<T>, val),
            "Incorrect struct layout offsets. Internal WutEngine error"
        );

        let layout = Layout::new::<LayoutHelper<T>>().pad_to_align();

        Self {
            component_id: T::get_component_id(),
            unpadded_component_size: size_of::<T>(),
            elem_layout: layout,
            drop_elem_fn: dynamic_drop::<T>,
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
            let new_elem_base_ptr = alloc.as_ptr().byte_add(self.elem_layout.size() * self.len);
            let entity_id_ptr = new_elem_base_ptr;
            let component_val_ptr = entity_id_ptr.byte_add(size_of::<EntityId>());

            *(entity_id_ptr as *mut EntityId) = entity;
            std::ptr::copy_nonoverlapping(as_raw, component_val_ptr, self.unpadded_component_size);
        }

        self.entity_to_idx.insert(entity, self.len);

        self.len += 1;
    }

    fn ensure_capacity(&mut self, required: usize) {
        if self.capacity >= required {
            return;
        }

        let alloc_ptr = unsafe {
            std::alloc::alloc(
                Layout::from_size_align(
                    self.elem_layout.size() * required,
                    self.elem_layout.align(),
                )
                .unwrap(),
            )
        };

        let new_alloc = NonNull::new(alloc_ptr).unwrap();

        if let Some(existing_alloc) = self.alloc.take() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    existing_alloc.as_ptr(),
                    new_alloc.as_ptr(),
                    self.elem_layout.size() * self.len,
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
                    let actual_ptr = alloc.as_ptr().byte_add(self.elem_layout.size() * i);
                    (self.drop_elem_fn)(actual_ptr as *mut ())
                }

                let layout = Layout::from_size_align(
                    self.elem_layout.size() * self.capacity,
                    self.elem_layout.align(),
                )
                .unwrap();

                std::alloc::dealloc(alloc.as_ptr(), layout);
            }

            self.capacity = 0;
        }
    }
}

unsafe fn dynamic_drop<T>(raw: *mut ()) {
    ptr::drop_in_place::<LayoutHelper<T>>(raw as *mut LayoutHelper<T>);
}

impl Drop for ComponentArray {
    fn drop(&mut self) {
        unsafe {
            self.dealloc_current_alloc();
        }
    }
}
