#[doc(hidden)]
#[inline(always)] // This was #[inline(never)]. Any reason?
pub(crate) fn clean_function_name(name: &str) -> &str {
    const USELESS_SCOPE_NAME_SUFFIX: &str = "::__f";

    let Some(name) = name.strip_suffix(USELESS_SCOPE_NAME_SUFFIX) else {
        return name;
    };

    name
    // // Remove any additional trailing suffixes
    // shorten_rust_function_name(name.trim_end_matches(USELESS_CLOSURE_SUFFIX))
}

#[doc(hidden)]
#[inline(always)]
pub(crate) fn type_name_of<T>(_: T) -> &'static str {
    core::any::type_name::<T>()
}
/// Returns the name of the calling function without a long module path prefix
///
/// Adapted from [puffin](https://github.com/EmbarkStudios/puffin/blob/c5276b9d5264af37a9c9fb2655990a3a0b720a0b/puffin/src/lib.rs#L178-L183)
macro_rules! current_function_name {
    () => {{
        fn __f() {}
        let name = $crate::util::type_name_of(__f);
        $crate::util::clean_function_name(name)
    }};
}

pub(crate) use current_function_name;
