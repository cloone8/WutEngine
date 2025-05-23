//! Viewport rendering functions

use std::sync::Mutex;

use wutengine_graphics::renderer::Viewport;

pub(in crate::graphics) static VIEWPORT_QUEUE: Mutex<Vec<Viewport>> = Mutex::new(Vec::new());

#[profiling::function]
pub(crate) fn queued_viewports() -> Vec<Viewport> {
    let locked = VIEWPORT_QUEUE.lock().unwrap();

    locked.clone()
}

#[profiling::function]
pub(crate) fn clear_queued_viewports() {
    let mut locked = VIEWPORT_QUEUE.lock().unwrap();

    locked.clear();
}
