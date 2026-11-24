#![doc = include_str!("../README.md")]

#[doc(inline)]
pub use wutengine_log as log;
mod executor;
pub mod graphics;
mod panic_handling;
pub mod profiling;
pub mod runtime;
pub mod time;
pub mod window;
pub use wutengine_macro::main;
#[doc(inline)]
pub use wutengine_math as math;
