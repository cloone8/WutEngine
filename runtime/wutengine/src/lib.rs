//! The main WutEngine runtime
//!
//! For more information and examples, see the crates.io page and the repository

extern crate alloc;

pub mod builtins;
pub mod component;
pub mod entity;
pub mod graphics;
pub mod profiling;
pub mod runtime;
pub mod system;
pub mod window;
pub mod world;

#[cfg(feature = "development_overlay")]
pub mod development_overlay;

#[doc(inline)]
pub use hecs;

#[doc(inline)]
pub use uuid;

pub mod log {
    //! Logging

    #[doc(inline)]
    pub use log::*;

    #[doc(inline)]
    pub use wutengine_logger::*;
}

#[doc(inline)]
pub use wutengine_assets as asset;

#[doc(inline)]
pub use wutengine_asset_server as asset_server;

#[doc(inline)]
pub use wutengine_audio as audio;

#[doc(inline)]
pub use wutengine_math as math;

#[doc(inline)]
pub use wutengine_input as input;

#[doc(inline)]
pub use wutengine_time as time;

#[doc(inline)]
pub use wutengine_physics as physics;

#[doc(inline)]
pub use wutengine_config as config;

#[doc(inline)]
pub use wutengine_event as event;

#[doc(inline)]
pub use wutengine_task as task;

pub mod util {
    //! Utility functions

    /// Combine two UUIDs into one. Usable in const contexts
    pub const fn combine_uuid(a: uuid::NonNilUuid, b: uuid::NonNilUuid) -> uuid::NonNilUuid {
        let bytes_a = a.get().into_bytes();
        let bytes_b = b.get().into_bytes();

        let bytes = [
            bytes_a[0].wrapping_mul(bytes_b[0]),
            bytes_a[1].wrapping_mul(bytes_b[1]),
            bytes_a[2].wrapping_mul(bytes_b[2]),
            bytes_a[3].wrapping_mul(bytes_b[3]),
            bytes_a[4].wrapping_mul(bytes_b[4]),
            bytes_a[5].wrapping_mul(bytes_b[5]),
            bytes_a[6].wrapping_mul(bytes_b[6]),
            bytes_a[7].wrapping_mul(bytes_b[7]),
            bytes_a[8].wrapping_mul(bytes_b[8]),
            bytes_a[9].wrapping_mul(bytes_b[9]),
            bytes_a[10].wrapping_mul(bytes_b[10]),
            bytes_a[11].wrapping_mul(bytes_b[11]),
            bytes_a[12].wrapping_mul(bytes_b[12]),
            bytes_a[13].wrapping_mul(bytes_b[13]),
            bytes_a[14].wrapping_mul(bytes_b[14]),
            bytes_a[15].wrapping_mul(bytes_b[15]),
        ];

        uuid::NonNilUuid::new(uuid::Builder::from_custom_bytes(bytes).into_uuid()).unwrap()
    }
}
