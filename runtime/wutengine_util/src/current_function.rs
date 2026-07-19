//! Utility module containing a macro (and some helper functions) for getting the current function name
//!
//! The whole thing is copied and adapted from [this puffin macro][puffin_code].
//!
//! [puffin] is licensed under the [MIT license](https://github.com/EmbarkStudios/puffin/blob/c5276b9d5264af37a9c9fb2655990a3a0b720a0b/LICENSE-MIT)
//!
//! [puffin]: https://github.com/EmbarkStudios/puffin/blob/c5276b9d5264af37a9c9fb2655990a3a0b720a0b
//! [puffin_code]: https://github.com/EmbarkStudios/puffin/blob/c5276b9d5264af37a9c9fb2655990a3a0b720a0b/puffin/src/lib.rs#L178-L183

#[doc(hidden)]
#[inline] // This was #[inline(never)]. Any reason?
pub fn clean_function_name(name: &str) -> &str {
    const USELESS_SCOPE_NAME_SUFFIX: &str = "::__f";

    let Some(name) = name.strip_suffix(USELESS_SCOPE_NAME_SUFFIX) else {
        return name;
    };

    name
}

#[doc(hidden)]
#[inline]
pub fn type_name_of<T>(_: T) -> &'static str {
    core::any::type_name::<T>()
}

/// Returns the name of the calling function without a long module path prefix
#[macro_export]
macro_rules! current_function_name {
    () => {{
        fn __f() {}
        let name = $crate::type_name_of(__f);
        $crate::clean_function_name(name)
    }};
}
