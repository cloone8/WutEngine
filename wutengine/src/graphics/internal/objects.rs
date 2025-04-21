use std::sync::Mutex;

use super::RenderCommand;

pub(in crate::graphics) static OBJECT_QUEUE: Mutex<Vec<RenderCommand>> = Mutex::new(Vec::new());

#[profiling::function]
pub(crate) fn queued_objects() -> Vec<RenderCommand> {
    let locked = OBJECT_QUEUE.lock().unwrap();

    locked.clone()
}

#[profiling::function]
pub(crate) fn clear_queued_objects() {
    let mut locked = OBJECT_QUEUE.lock().unwrap();

    locked.clear();
}
