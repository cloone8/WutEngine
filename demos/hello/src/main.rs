//! Hello: A tiny demo that starts the logger and writes a log at each level, inside a profiler scope

use std::env;
use std::process::ExitCode;

use wutengine::log;
use wutengine::profiling;
use wutengine::runtime;
use wutengine::runtime::StartupOptions;

fn main() -> ExitCode {
    profiling::function_scope!();

    runtime::start(&StartupOptions {
        org_name: "wutengine",
        app_name: "demo_hello",
    });

    log::info!("Hello, world!");

    let mut do_warn = false;
    let mut do_err = false;
    let mut num_args = 0;

    for arg in env::args() {
        log::trace!("Checking arg: {arg}");

        num_args += 1;

        let lowercase = arg.trim().to_lowercase();

        if lowercase.contains("warn") {
            do_warn = true;
        }

        if lowercase.contains("err") {
            do_err = true;
        }
    }

    log::debug!("Counted {num_args} args");

    if do_warn {
        log::warn!("Warning requested");
    }

    if do_err {
        log::error!("Error requested, returning failure exit code");
        ExitCode::FAILURE
    } else {
        log::info!("Program succesful");
        ExitCode::SUCCESS
    }
}
