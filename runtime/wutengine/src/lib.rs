#![doc = include_str!("../README.md")]

#[doc(inline)]
pub use wutengine_log as log;
mod executor;
mod panic_handling;
pub mod profiling;
mod renderer;
pub mod runtime;

pub use wutengine_macro::main;
