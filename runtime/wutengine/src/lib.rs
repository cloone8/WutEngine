#![doc = include_str!("../README.md")]

#[doc(inline)]
pub use wutengine_log as log;
mod executor;
pub mod graphics;
mod panic_handling;
mod platform;
pub mod profiling;
pub mod runtime;
pub mod time;
pub mod window;
pub use wutengine_macro::main;
#[doc(inline)]
pub use wutengine_math as math;

mod build_config_checks {
    //! Build config checks. Should not contain logic

    #[cfg(all(target_arch = "wasm32", not(feature = "interactive")))]
    compile_error!("Web builds cannot be headless");
}
