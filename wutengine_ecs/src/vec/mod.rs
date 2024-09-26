use core::alloc::Layout;
use std::any::{Any, TypeId};
use std::ptr::NonNull;

#[cfg(test)]
mod test;

mod dynamic;

pub use dynamic::*;

#[derive(Debug)]
pub struct AnyVec {
    len: usize,
    capacity: usize,
    storage: Option<NonNull<u8>>,
    base_layout: Layout,
    actual_type: TypeId,
    drop_fn: unsafe fn(*mut u8),
}

#[derive(Debug, Clone, Copy)]
pub struct AnyVecStorageDescriptor {
    base_layout: Layout,
    actual_type: TypeId,
    drop_fn: unsafe fn(*mut u8),
}

impl AnyVecStorageDescriptor {
    pub fn new<T: Any>() -> Self {
        debug_assert_ne!(
            TypeId::of::<Dynamic>(),
            TypeId::of::<T>(),
            "Accidentally tried to create AnyVec for Dynamic"
        );

        Self {
            base_layout: Layout::new::<T>(),
            actual_type: TypeId::of::<T>(),
            drop_fn: |x| {
                unsafe { std::ptr::drop_in_place(x as *mut T) };
            },
        }
    }
}

impl AnyVec {
    pub fn new<T: Any>() -> Self {
        debug_assert_ne!(
            TypeId::of::<Dynamic>(),
            TypeId::of::<T>(),
            "Accidentally tried to create AnyVec for Dynamic"
        );

        Self::from_descriptor(AnyVecStorageDescriptor::new::<T>())
    }

    pub fn from_descriptor(descriptor: AnyVecStorageDescriptor) -> Self {
        Self {
            len: 0,
            capacity: 0,
            storage: None,
            base_layout: descriptor.base_layout,
            actual_type: descriptor.actual_type,
            drop_fn: descriptor.drop_fn,
        }
    }

    pub fn get_descriptor(&self) -> AnyVecStorageDescriptor {
        AnyVecStorageDescriptor {
            base_layout: self.base_layout,
            actual_type: self.actual_type,
            drop_fn: self.drop_fn,
        }
    }

    pub fn with_capacity<T: Any>(num: usize) -> Self {
        debug_assert_ne!(
            TypeId::of::<Dynamic>(),
            TypeId::of::<T>(),
            "Accidentally tried to create AnyVec for Dynamic"
        );

        let mut new = Self::new::<T>();

        new.ensure_capacity(num);

        new
    }

    pub fn duplicate_for_type(&self) -> Self {
        Self {
            len: 0,
            capacity: 0,
            storage: None,
            base_layout: self.base_layout,
            actual_type: self.actual_type,
            drop_fn: self.drop_fn,
        }
    }

    pub fn inner_type_id(&self) -> TypeId {
        self.actual_type
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push<T: Any>(&mut self, val: T) {
        assert_eq!(self.actual_type, TypeId::of::<T>(), "Wrong type given!");

        self.ensure_capacity(self.len + 1);

        let storage_ptr = self
            .storage
            .expect("Storage should have been allocated")
            .as_ptr() as *mut T;

        unsafe { storage_ptr.add(self.len).write(val) };

        self.len += 1;
    }

    #[must_use]
    pub fn pop<T: Any>(&mut self) -> Option<T> {
        assert_eq!(self.actual_type, TypeId::of::<T>(), "Wrong type given!");

        if self.len == 0 {
            return None;
        }

        debug_assert!(self.len <= self.capacity);

        let storage_ptr = self.storage.expect("Len > 0 but no storage").as_ptr() as *mut T;

        let val: T = unsafe { storage_ptr.add(self.len - 1).read() };

        self.len -= 1;

        Some(val)
    }

    pub fn swap_remove(&mut self, index: usize) {
        assert!(
            index < self.len(),
            "Index out of range: {} (max {})",
            index,
            self.len()
        );

        let storage_ptr = self.storage.expect("Len > 0 but no storage").as_ptr();

        unsafe {
            let elem_ptr = storage_ptr.byte_add(self.base_layout.size() * index);

            debug_assert_eq!(0, elem_ptr as usize % self.base_layout.align());

            // First drop the old element
            (self.drop_fn)(elem_ptr);

            // Then replace it with the one at the tail
            self.overwrite_with_tail(index);

            if index != (self.len() - 1) {
                let tail_elem_ptr =
                    storage_ptr.byte_add(self.base_layout.size() * (self.len() - 1));

                debug_assert_ne!(elem_ptr, tail_elem_ptr);

                elem_ptr.copy_from_nonoverlapping(tail_elem_ptr, self.base_layout.size());
            }
        }

        self.len -= 1;
    }

    pub fn take_from_other(&mut self, other: &mut Self, index: usize) {
        assert_eq!(self.actual_type, other.actual_type, "Non-matching types!");
        assert!(
            index < other.len(),
            "Index out of range: {} (max {})",
            index,
            other.len()
        );
        debug_assert_eq!(
            self.base_layout, other.base_layout,
            "Types of AnyVecs are the same, but layouts aren't"
        );

        self.ensure_capacity(self.len() + 1);

        let other_storage_ptr = other.storage.expect("Len > 0 but no storage").as_ptr();
        let my_storage_ptr = self.storage.expect("Len > 0 but no storage").as_ptr();

        unsafe {
            let source_ptr = other_storage_ptr.byte_add(other.base_layout.size() * index);
            let dst_ptr = my_storage_ptr.byte_add(self.base_layout.size() * self.len());

            debug_assert_eq!(0, source_ptr as usize % other.base_layout.align());
            debug_assert_eq!(0, dst_ptr as usize % self.base_layout.align());

            // First copy the element from `other` to `self`
            source_ptr.copy_to_nonoverlapping(dst_ptr, other.base_layout.size());

            // Now overwrite the copied (moved) element in `other` with the tail of `other`
            other.overwrite_with_tail(index);
        }

        self.len += 1;
        other.len -= 1;
    }

    /// Overwrites the element at the given index with the current tail, _without_ running
    /// its drop function and _without_ modifying the length of `self`.
    ///
    /// If the given index _is_ the tail, does nothing.
    unsafe fn overwrite_with_tail(&mut self, index: usize) {
        let elem_size = self.base_layout.size();
        let storage_ptr = self.storage.expect("Len > 0 but no storage").as_ptr();
        let to_overwrite_ptr: *mut u8 = storage_ptr.byte_add(elem_size * index);

        if index != (self.len() - 1) {
            let tail_elem_ptr: *mut u8 = storage_ptr.byte_add(elem_size * (self.len() - 1));

            debug_assert_ne!(to_overwrite_ptr, tail_elem_ptr);

            to_overwrite_ptr.copy_from_nonoverlapping(tail_elem_ptr, elem_size);
        }
    }

    pub fn clear(&mut self) {
        if self.capacity > 0 {
            let storage_ptr = self.storage.expect("Capacity > 0 but no storage").as_ptr();

            unsafe {
                for i in 0..self.len {
                    let byte_offset = self.base_layout.size() * i;
                    let elem_ptr = storage_ptr.byte_add(byte_offset);

                    debug_assert_eq!(0, elem_ptr as usize % self.base_layout.align());

                    (self.drop_fn)(elem_ptr);
                }
            }
        } else {
            debug_assert_eq!(0, self.len);
        }

        self.len = 0;
    }

    fn ensure_capacity(&mut self, at_least: usize) {
        if self.capacity >= at_least {
            return;
        }

        if self.base_layout.size() == 0 {
            if self.storage.is_none() {
                self.storage = Some(NonNull::<u8>::dangling())
            }

            self.capacity = at_least;

            return;
        }

        let new_layout = array_layout(self.base_layout, at_least);

        if let Some(cur_alloc) = self.storage {
            let cur_layout = array_layout(self.base_layout, self.capacity);

            assert!(cur_layout.size() < new_layout.size());

            self.storage = Some(
                NonNull::new(unsafe {
                    std::alloc::realloc(cur_alloc.as_ptr(), cur_layout, new_layout.size())
                })
                .expect("std::alloc::realloc returned nullptr"),
            );
        } else {
            self.storage = Some(
                NonNull::new(unsafe { std::alloc::alloc(new_layout) })
                    .expect("std::alloc::alloc returned nullptr"),
            );
        }

        self.capacity = at_least;
    }

    pub fn as_slice<T: Any>(&self) -> &[T] {
        assert_eq!(self.actual_type, TypeId::of::<T>(), "Wrong type given!");

        if let Some(storage) = self.storage {
            debug_assert!(self.len <= self.capacity);
            unsafe { std::slice::from_raw_parts(storage.as_ptr() as *const T, self.len) }
        } else {
            debug_assert_eq!(0, self.len);
            unsafe { std::slice::from_raw_parts(NonNull::dangling().as_ptr(), 0) }
        }
    }

    pub fn as_mut_slice<T: Any>(&mut self) -> &mut [T] {
        assert_eq!(self.actual_type, TypeId::of::<T>(), "Wrong type given!");

        if let Some(storage) = self.storage {
            debug_assert!(self.len <= self.capacity);
            unsafe { std::slice::from_raw_parts_mut(storage.as_ptr() as *mut T, self.len) }
        } else {
            debug_assert_eq!(0, self.len);
            unsafe { std::slice::from_raw_parts_mut(NonNull::dangling().as_ptr(), 0) }
        }
    }
}

impl Drop for AnyVec {
    fn drop(&mut self) {
        if let Some(storage) = self.storage {
            unsafe {
                let storage_ptr = storage.as_ptr();

                for i in 0..self.len {
                    let byte_offset = self.base_layout.size() * i;
                    let elem_ptr = storage_ptr.byte_add(byte_offset);

                    debug_assert_eq!(0, elem_ptr as usize % self.base_layout.align());

                    (self.drop_fn)(elem_ptr);
                }

                if self.base_layout.size() != 0 {
                    std::alloc::dealloc(storage_ptr, array_layout(self.base_layout, self.capacity))
                }
            }
        }
    }
}

fn array_layout(base: Layout, num: usize) -> Layout {
    Layout::from_size_align(base.size() * num, base.align()).unwrap_or_else(|_| {
        panic!(
            "Cannot construct layout for base {:?} with {} elements",
            base, num
        )
    })
}
