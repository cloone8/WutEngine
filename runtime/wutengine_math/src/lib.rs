#![doc = include_str!("../README.md")]
#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub use glam::*;

mod color;

pub use color::*;
