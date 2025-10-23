//! WutEngine math functions and types.
//! Many of the basic linear algebra types are re-exports from [glam]

use core::ops::{Add, Sub};
use std::process::Output;

pub use glam::*;

/// Unclamped linear interpolation between A-B
#[inline(always)]
pub fn lerp<N, T>(a: N, b: N, t: T) -> N
where
    N: Sub<Output = N> + Add<Output = N> + Copy,
    T: std::ops::Mul<N, Output = N>,
{
    a + t * (b - a)
}
