//! Physics and collision related components

mod physics2d;
mod physics3d;

pub use physics2d::*;

#[expect(unused_imports, unreachable_pub)]
pub use physics3d::*;
