#![doc = include_str!("../../README.md")]

extern crate alloc;

pub mod builtins;
pub mod color;
pub mod component;
pub mod config;
pub mod entity;
pub mod graphics;
pub mod math;
pub mod profiling;
pub mod runtime;
pub mod system;
pub mod thread;
pub mod time;
pub(crate) mod util;
pub mod window;
pub mod world;

#[doc(inline)]
pub use hecs;

#[doc(inline)]
pub use log;

#[doc(inline)]
pub use wgpu;

#[doc(inline)]
pub use wutengine_asset as asset;
