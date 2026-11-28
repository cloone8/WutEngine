//! Window: A tiny demo that simply opens a gray window

use wutengine::profiling;

#[wutengine::main(org = "wutengine", app = "demo_window")]
fn main() {
    profiling::function_scope!();
}
