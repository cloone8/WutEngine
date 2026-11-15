//! Engine runtime lifecycle

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use wutengine_log::FileLogger;
use wutengine_log::TerminalLogger;

use crate::log;

/// Startup options used to start the [WutEngine runtime](crate::runtime)
#[derive(Debug)]
pub struct StartupOptions<'a> {
    /// Organization name, used for storing/loading data
    pub org_name: &'a str,

    /// Application name, used for storing/loading data
    pub app_name: &'a str,
}

/// Starts the WutEngine runtime. Must be called once, from the main thread
pub fn start(options: &StartupOptions) {
    static INIT: AtomicBool = AtomicBool::new(false);

    if INIT.swap(true, Ordering::Relaxed) {
        panic!("Runtime already initialized")
    }

    log::init(log::LogSettings::new(), vec![Box::new(TerminalLogger)]);

    match FileLogger::new(options.org_name, options.app_name, "engine") {
        Ok(fl) => log::add_output(Box::new(fl)),
        Err(e) => {
            log::error!("Failed to create file logger: {e}");
        }
    }

    log::info!("WutEngine start-up done");
}
