use core::alloc::Layout;
use std::any::{Any, TypeId};
use std::ptr::NonNull;

#[cfg(test)]
mod test;

pub struct AnyVec {
    len: usize,
    capacity: usize,
    storage: Option<NonNull<u8>>,
    base_layout: Layout,
    actual_type: TypeId,
    drop_fn: unsafe fn(*mut u8),
}

impl AnyVec {
    pub fn new<T: Any>() -> Self {
        Self {
            len: 0,
            capacity: 0,
            storage: None,
            base_layout: Layout::new::<T>(),
            actual_type: TypeId::of::<T>(),
            drop_fn: |x| {
                unsafe { std::ptr::drop_in_place(x as *mut T) };
            },
        }
    }

    pub fn with_capacity<T: Any>(num: usize) -> Self {
        let mut new = Self::new::<T>();

        new.ensure_capacity(num);

        new
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

                std::alloc::dealloc(storage_ptr, array_layout(self.base_layout, self.capacity))
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
