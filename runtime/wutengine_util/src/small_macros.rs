//! Small utility macros

/// Creates a hashmap and inserts the given keys and values. [`Into::into`] is
/// called on each key and value before it is inserted.
///
/// Used like:
/// ```
/// use std::collections::HashMap;
///
/// let new_map: HashMap<String, i32> = wutengine_util::map![
///     "a" => 1,
///     "b" => 2
/// ];
/// ```
#[macro_export]
macro_rules! map {
    () => {
        ::std::collections::HashMap::default()
    };

    ($($key:expr => $val:expr),+) => {{
        let mut new_hashmap = ::std::collections::HashMap::default();

        $(
            new_hashmap.insert($key.into(), $val.into());
        )*

        new_hashmap
    }};
}

/// Macro that marks the current spot as unreachable. Checked in debug builds,
/// unchecked in release builds.
#[macro_export]
macro_rules! unreachable_dbg {
    ($($arg:tt)*) => {{
        // Dummy unsafe no-op to force unsafe{} around this macro
        #[allow(clippy::useless_transmute, reason = "Dummy op")]
        {
        _ = ::core::mem::transmute::<(), ()>(());
        }

        #[cfg(debug_assertions)]
        unreachable!($($arg)*);

        #[cfg(not(debug_assertions))]
        ::core::hint::unreachable_unchecked();
    }};
}

/// Logs at the given level, but only once. Same syntax as [`log::log`]
#[macro_export]
macro_rules! log_once {
    ($level:expr, $($arg:tt)*) => {{
        static LOG_ONCE: ::std::sync::Once = ::std::sync::Once::new();

        LOG_ONCE.call_once(|| {
            ::log::log!($level, $($arg)*);
        });
    }};
}

/// Shorthand for [log_once] with level [`log::Level::Trace`]
#[macro_export]
macro_rules! trace_once {
    ($($arg:tt)*) => {
        $crate::log_once!(::log::Level::Trace, $($arg)*);
    };
}

/// Shorthand for [log_once] with level [`log::Level::Debug`]
#[macro_export]
macro_rules! debug_once {
    ($($arg:tt)*) => {
        $crate::log_once!(::log::Level::Debug, $($arg)*);
    };
}

/// Shorthand for [log_once] with level [`log::Level::Info`]
#[macro_export]
macro_rules! info_once {
    ($($arg:tt)*) => {
        $crate::log_once!(::log::Level::Info, $($arg)*);
    };
}

/// Shorthand for [log_once] with level [`log::Level::Warn`]
#[macro_export]
macro_rules! warn_once {
    ($($arg:tt)*) => {
        $crate::log_once!(::log::Level::Warn, $($arg)*);
    };
}

/// Shorthand for [log_once] with level [`log::Level::Error`]
#[macro_export]
macro_rules! error_once {
    ($($arg:tt)*) => {
        $crate::log_once!(::log::Level::Error, $($arg)*);
    };
}
