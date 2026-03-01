//! Small utility macros

/// Creates a hashmap and inserts the given keys and values. [Into::into] is
/// called on each key and value before it is inserted.
///
/// Used like:
/// ```
/// use std::collections::HashMap;
///
/// let new_map: HashMap<String, i32> = map![
///     "a" => 1,
///     "b" => 2
/// ];
/// ```
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

pub(crate) use map;

// /// Macro that marks the current spot as unreachable. Checked in debug builds,
// /// unchecked in release builds.
// macro_rules! unreachable_dbg {
//     ($($arg:tt)*) => {{
//         // Dummy unsafe no-op to force unsafe{} around this macro
//         #[allow(clippy::useless_transmute, reason = "Dummy op")]
//         {
//         _ = ::core::mem::transmute::<(), ()>(());
//         }

//         #[cfg(debug_assertions)]
//         unreachable!($($arg)*);

//         #[cfg(not(debug_assertions))]
//         ::core::hint::unreachable_unchecked();
//     }};
// }

// pub(crate) use unreachable_dbg;
