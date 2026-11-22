//! Main in-frame render functions

use wutengine_sync::thread_context;

/// Renders the current state of the renderer
pub fn render() {
    profiling::function_scope!();
    thread_context::assert_main_context();
}
