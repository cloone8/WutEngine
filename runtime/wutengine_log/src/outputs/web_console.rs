use web_sys::console;
use web_sys::wasm_bindgen::JsValue;

use crate::LoggerOutput;

/// Logger that writes to the browser console
#[derive(Debug, Clone, Copy, Default)]
pub struct BrowserConsoleLogger;

impl BrowserConsoleLogger {
    /// Returns a new [`BrowserConsoleLogger`]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl LoggerOutput for BrowserConsoleLogger {
    fn write(&self, entry: &crate::LogEntry) {
        let msg = JsValue::from_str(&format!("({}) {}", entry.category, entry.message));

        match entry.level {
            log::Level::Error => console::error_1(&msg),
            log::Level::Warn => console::warn_1(&msg),
            log::Level::Info => console::info_1(&msg),
            log::Level::Debug => console::debug_1(&msg),
            log::Level::Trace => console::debug_1(&msg), // Not trace because that dumps the entire stacktrace
        }
    }
}
