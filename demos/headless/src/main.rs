//! Headless: A demo showing a headless WutEngine build

use wutengine::log;
use wutengine::profiling;

#[wutengine::main(org = "wutengine", app = "demo_headless")]
fn main() {
    profiling::function_scope!();

    log::info!("Hello, headless world!");
}
