macro_rules! debug_assert_aligned {
    ($ptr:expr, $align:expr) => {
        if cfg!(debug_assertions) {
            $crate::storage::ptr_helpers::assert_aligned!($ptr, $align);
        }
    };
}

pub(super) use debug_assert_aligned;

macro_rules! assert_aligned {
    ($ptr:expr, $align:expr) => {
        assert!(
            $crate::storage::ptr_helpers::is_aligned_to($ptr, $align),
            "Alignment error. Expected at least {}, actual max alignment {}",
            $align,
            $crate::storage::ptr_helpers::calc_max_alignment($ptr)
        )
    };
}

pub(super) use assert_aligned;

pub fn is_aligned_to<T>(ptr: *const T, align: usize) -> bool {
    (ptr as usize) % align == 0
}

pub fn calc_max_alignment<T>(ptr: *const T) -> usize {
    let mut cur_align: usize = 1;

    while (cur_align * 2) < usize::MAX {
        if (ptr as usize) % (cur_align * 2) != 0 {
            return cur_align;
        }

        cur_align *= 2;
    }

    unreachable!("Always at least alignment of 1");
}
