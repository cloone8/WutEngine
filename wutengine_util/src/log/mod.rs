//! Logging macros and utilities

/// Writes the given warning exactly once
#[macro_export]
macro_rules! warn_once {
    ($($tokens:tt)*) => {{
        static WARNED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);

        if !WARNED.swap(true, ::core::sync::atomic::Ordering::AcqRel) {
            ::log::warn!($($tokens)*);
        }
    }};
}

/// Writes the given error exactly once
#[macro_export]
macro_rules! err_once {
    ($($tokens:tt)*) => {{
        static ERRD: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);

        if !ERRD.swap(true, ::core::sync::atomic::Ordering::AcqRel) {
            ::log::error!($($tokens)*);
        }
    }};
}
