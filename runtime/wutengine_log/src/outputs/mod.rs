//! Various builtin log outputs

#[cfg(feature = "file_output")]
mod file;

#[cfg(feature = "file_output")]
pub use file::*;

#[cfg(feature = "terminal_output")]
mod terminal;

#[cfg(feature = "terminal_output")]
pub use terminal::*;

#[cfg(target_arch = "wasm32")]
mod web_console;

#[cfg(target_arch = "wasm32")]
pub use web_console::*;
