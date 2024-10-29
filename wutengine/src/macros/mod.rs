//! Macros that make working with WutEngine easier

pub use wutengine_macro::*;

/// Creates a hashmap and inserts the given keys and values. [Into::into] is
/// called on each key and value before it is inserted.
///
/// Used like:
/// ```
/// use wutengine::map;
/// use std::collections::HashMap;
///
/// let new_map: HashMap<String, i32> = map![
///     "a" => 1,
///     "b" => 2
/// ];
/// ```
#[macro_export]
macro_rules! map {
    ($($key:expr => $val:expr),+) => {{
        let mut new_hashmap = ::std::collections::HashMap::default();

        $(
            new_hashmap.insert($key.into(), $val.into());
        )*

        new_hashmap
    }};
}
