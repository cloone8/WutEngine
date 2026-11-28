//! Headless: A demo showing a headless WutEngine build

use wutengine::log;
use wutengine::profiling;
use wutengine::runtime;

#[wutengine::main(org = "wutengine", app = "demo_headless")]
fn main() {
    profiling::function_scope!();

    log::info!("Hello, headless world!");

    runtime::hooks::register_started_hook(|| {
        log::info!("In `started` callback, quitting now");

        runtime::quit();
    });

    runtime::hooks::register_quitting_hook(|| {
        log::info!("In `quitting` hook, goodbye!");
    });
}
