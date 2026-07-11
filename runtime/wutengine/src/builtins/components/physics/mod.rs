//! Physics components

#[cfg(feature = "phys2d")]
mod collider2d;
#[cfg(feature = "phys2d")]
pub use collider2d::*;

#[cfg(feature = "phys3d")]
mod collider3d;
#[cfg(feature = "phys3d")]
pub use collider3d::*;
