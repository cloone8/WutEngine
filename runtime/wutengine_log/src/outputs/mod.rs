//! Various builtin log outputs

#[cfg(feature = "file_output")]
mod file;

#[cfg(feature = "file_output")]
pub use file::FileLogger;

#[cfg(feature = "terminal_output")]
mod terminal;

#[cfg(feature = "terminal_output")]
pub use terminal::TerminalLogger;

#[cfg(target_arch = "wasm32")]
mod web_console;
