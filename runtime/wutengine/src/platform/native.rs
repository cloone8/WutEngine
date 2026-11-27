//! Native (desktop) platform implementation

use std::process;

use wutengine_log::FileLogger;
use wutengine_log::LoggerOutput;

use crate::log;
use crate::runtime;
use crate::runtime::StartupOptions;

/// Return type of the engine entry point. Native runs the engine to completion and exits the
/// process from there, so the entry point never returns
pub(super) type StartRet = !;

/// Platform entry point, called from the game's `main`. Runs the engine, then exits the process
pub(super) fn main(options: StartupOptions, app_entrypoint: impl FnOnce() + 'static) -> StartRet {
    runtime::run(&options, app_entrypoint);
    runtime::exit_now(0);
}

/// The log output that is available from the very start: the terminal
pub(super) fn base_log_output() -> Box<dyn LoggerOutput> {
    Box::new(wutengine_log::TerminalLogger::new(
        wutengine_log::OutputMode::Stdout,
        wutengine_log::ColorMode::Auto,
    ))
}

/// Adds the log outputs that need the logger to be running already, so that failures can be logged.
/// On native, this is the log file
pub(super) fn add_log_outputs(options: &StartupOptions) {
    match FileLogger::new(options.org_name, options.app_name, "engine") {
        Ok(fl) => log::add_output(Box::new(fl)),
        Err(e) => {
            log::error!("Failed to create file logger: {e}");
        }
    }
}

/// Exits the process with the given exit code
pub(super) fn exit(code: i32) -> ! {
    process::exit(code);
}
