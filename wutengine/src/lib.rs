//! The root of the WutEngine game engine. Use this crate as a dependency when building WutEngine games, at it
//! re-exports all relevant subcrates.

#[doc(inline)]
pub use wutengine_core::assert;

#[doc(inline)]
pub use wutengine_core::profiling;

#[doc(inline)]
pub use wutengine_graphics as graphics;

mod embedded {
    //! Embedded resources. Will probably be replaced with something more intelligent later

    use include_dir::{include_dir, Dir};

    /// Embedded shader sources. Will be replaced with a more sophisticated shader loading system later.
    pub(crate) static SHADERS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/shaders");
}

pub mod asset;
pub mod builtins;
pub mod component;
pub mod context;
pub mod gameobject;
pub mod input;
pub mod log;
pub mod macros;
pub mod math;
pub mod plugins;
pub mod renderer;
pub mod runtime;
pub mod time;
pub(crate) mod util;
pub mod windowing;

/// For use in engine plugins
pub use winit;

//NOTE: This top-level module will _not_ be logged due to level filtering difficulties. Put any logic in a submodule.
