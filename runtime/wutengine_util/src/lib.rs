#![doc = include_str!("../README.md")]

mod current_function;
mod init_once;
mod main_thread;
mod main_thread_only;
mod shard_hasher;
mod small_macros;

#[cfg(feature = "hashbrown")]
mod intmap;

pub use current_function::*;
pub use init_once::*;
#[cfg(feature = "hashbrown")]
pub use intmap::*;
pub use main_thread::*;
pub use main_thread_only::*;
pub use shard_hasher::*;

#[cfg(feature = "hashbrown")]
pub use hashbrown;
