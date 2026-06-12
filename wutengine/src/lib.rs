#![doc = include_str!("../../README.md")]

extern crate alloc;

pub mod builtins;
pub mod component;
pub mod entity;
pub mod graphics;
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
pub use wutengine_asset as asset;

#[doc(inline)]
pub use wutengine_audio as audio;

#[doc(inline)]
pub use wutengine_math as math;

#[doc(inline)]
pub use wutengine_input as input;

#[doc(inline)]
pub use wutengine_time as time;

#[doc(inline)]
pub use wutengine_physics2d as physics2d;

#[doc(inline)]
pub use wutengine_physics3d as physics3d;

#[doc(inline)]
pub use wutengine_config as config;
