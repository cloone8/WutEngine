//! Asset handling

use core::any::Any;

/// Trait implemented by types that can be used as a WutEngine asset
pub trait Asset: Send + Sync + Any {}
