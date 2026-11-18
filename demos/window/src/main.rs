//! Hello: A tiny demo that starts the logger and writes a log at each level, inside a profiler scope

use wutengine::profiling;

#[wutengine::main(org = "wutengine", app = "demo_window")]
fn main() {
    profiling::function_scope!();

}


