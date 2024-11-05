//! Physics functionality. Both 2D and 3D.

use std::any::Any;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use crate::plugins::{Context, WutEnginePlugin};
use crate::time::Time;
use glam::Vec2;

pub(crate) mod physics2d;
pub(crate) mod physics3d;

pub use physics2d::*;
pub use physics3d::*;

#[doc(inline)]
pub use rapier2d as raw_2d;

#[doc(inline)]
pub use rapier3d as raw_3d;
