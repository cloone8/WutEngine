//! Hello: A tiny demo that starts the logger and writes a log at each level, inside a profiler scope

use std::env;

use wutengine::log;
use wutengine::profiling;
use wutengine::runtime;
use wutengine::runtime::StartupOptions;

#[wutengine::main(org = "wutengine", app = "demo_hello")]
fn main() {
    profiling::function_scope!();

    log::info!("Hello, world!");

    let mut do_warn = false;
    let mut do_err = false;
    let mut num_args = 0;

    for arg in env::args().skip(1) {
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
        runtime::quit(1);
    }

    log::info!("Program successful");
    runtime::quit(0);
}
