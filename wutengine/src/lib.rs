#![doc = include_str!("../../README.md")]

extern crate alloc;

pub mod builtins;
pub mod color;
pub mod component;
pub mod config;
pub mod entity;
pub mod graphics;
pub mod physics;
pub mod profiling;
pub mod runtime;
pub mod system;
pub mod thread;
pub mod window;
pub mod world;

#[cfg(feature = "development_overlay")]
pub mod development_overlay;

#[doc(inline)]
pub use hecs;

#[doc(inline)]
pub use log;

#[doc(inline)]
pub use wgpu;

#[doc(inline)]
pub use wutengine_asset as asset;

#[doc(inline)]
pub use wutengine_math as math;

#[doc(inline)]
pub use wutengine_input as input;

#[doc(inline)]
pub use wutengine_time as time;
