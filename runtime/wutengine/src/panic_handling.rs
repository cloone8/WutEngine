//! Panic handlers

use core::fmt;
use core::fmt::Write;
use std::backtrace::Backtrace;
use std::backtrace::BacktraceStatus;
use std::panic::PanicHookInfo;

use crate::log;

/// A basic panic hook that shows the message and dumps the stacktrace into the logs
pub(crate) fn panic_hook(info: &PanicHookInfo) {
    let location_fmt = fmt::from_fn(|f| match info.location() {
        Some(loc) => write!(f, "{}:{}:{}", loc.file(), loc.line(), loc.column()),
        None => write!(f, "<unknown>"),
    });

    let mut panic_msg = String::new();

    match info.payload_as_str() {
        Some(msg) => {
            _ = writeln!(panic_msg, "\nPANIC @ {location_fmt}: {msg}");
        }
        None => {
            _ = writeln!(panic_msg, "\nPANIC @ {location_fmt}");
        }
    }

    let backtrace = Backtrace::force_capture();

    if backtrace.status() == BacktraceStatus::Captured {
        _ = writeln!(panic_msg, "\n\n{backtrace}");
    } else {
        _ = writeln!(panic_msg, "<no backtrace available>");
    }

    log::error!("{panic_msg}");
    log::flush_on_panic();
}
