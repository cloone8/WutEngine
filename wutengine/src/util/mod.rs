//! Utility functions and macros

mod current_function;
mod init_once;
mod main_thread;
mod shard_hasher;
mod small_macros;

pub(crate) use current_function::*;
pub(crate) use init_once::*;
pub(crate) use main_thread::*;

#[expect(unused, reason = "Will be used for optimization later")]
pub(crate) use shard_hasher::*;

pub(crate) use small_macros::*;
