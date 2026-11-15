//! `xtask`-specific logger

use log::LevelFilter;
use log::Log;

/// Logger used by the `xtask` crate
#[derive(Debug, Clone, Copy)]
pub(crate) struct XTaskLogger;

impl XTaskLogger {
    /// Initializes the [`XTaskLogger`] as the global logger
    pub(crate) fn init() {
        log::set_logger(Box::leak(Box::new(Self))).expect("No logger should have been set");
        log::set_max_level(LevelFilter::Trace);
    }
}

impl Log for XTaskLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        _ = metadata;
        true
    }

    fn log(&self, record: &log::Record) {
        eprintln!("[XTASK] ({}) - {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
