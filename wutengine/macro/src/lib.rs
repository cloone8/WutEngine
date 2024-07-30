#[macro_export]
macro_rules! map {
    ($($key:expr => $val:expr),+) => {{
        let mut new_hashmap = ::std::collections::HashMap::new();

        $(
            new_hashmap.insert($key, $val);
        )*

        new_hashmap
    }};
}
