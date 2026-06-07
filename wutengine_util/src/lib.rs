#![doc = include_str!("../README.md")]

mod current_function;
mod init_once;
mod main_thread;
mod shard_hasher;
mod small_macros;

pub use current_function::*;
pub use init_once::*;
pub use main_thread::*;

pub use shard_hasher::*;
