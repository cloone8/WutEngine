/// For use in [wgpu] descriptors that want an optional label. Returns the label if debug assertions are enabled, and [None] otherwise
#[cfg(debug_assertions)]
macro_rules! debug_label {
    ($s:literal) => {
        Some($s)
    };

    ($($arg:tt)*) => {
        Some(format!($($arg)*).as_str())
    };
}

/// For use in [wgpu] descriptors that want an optional label. Returns the label if debug assertions are enabled, and [None] otherwise
#[cfg(not(debug_assertions))]
macro_rules! debug_label {
    ($s:literal) => {
        None
    };

    ($($arg:tt)*) => {
        None
    };
}

pub(crate) use debug_label;
