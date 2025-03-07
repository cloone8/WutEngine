//! The root of the WutEngine game engine. Use this crate as a dependency when building WutEngine games, at it
//! re-exports all relevant subcrates.

#[doc(inline)]
pub use wutengine_core::assert;

#[doc(inline)]
pub use wutengine_core::profiling;

#[doc(inline)]
pub use wutengine_graphics as graphics;

pub mod asset;
pub mod builtins;
pub mod component;
pub mod context;
pub mod gameobject;
pub mod global;
pub mod input;
pub mod log;
pub mod macros;
pub mod math;
pub mod physics;
pub mod plugins;
pub mod renderer;
pub mod runtime;
pub mod time;
pub(crate) mod util;
pub mod windowing;

/// For use in engine plugins
pub use winit;

//NOTE: This top-level module will _not_ be logged due to level filtering difficulties. Put any logic in a submodule.
