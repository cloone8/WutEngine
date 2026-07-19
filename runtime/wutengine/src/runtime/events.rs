//! Main runtime events

use alloc::sync::Arc;
use core::cell::RefCell;

use wutengine_util::MainThreadOnly;

use super::Runtime;

/// Event collector for events that should be handled when only the main [`Runtime`] is running.
/// Useful for when events need to alter the [`Runtime`] mutably
pub(super) static MAIN_RUNTIME_EVENTS: MainThreadOnly<RefCell<Vec<Arc<dyn MainRuntimeEvent>>>> =
    MainThreadOnly::new(RefCell::new(Vec::new()));

/// Sends an event to the main runtime events collector
fn send_to_main_runtime_collector<T: MainRuntimeEvent + Clone>(event: &T) {
    let cloned = Arc::new(event.clone()) as Arc<dyn MainRuntimeEvent>;

    MAIN_RUNTIME_EVENTS.borrow_mut().push(cloned);
}

pub(super) fn add_event_listeners(runtime: &mut Runtime) {
    _ = wutengine_event::subscribe::<AddOnExitHandler>(send_to_main_runtime_collector);
    _ = wutengine_event::subscribe::<AddOnExitRequestedHandler>(send_to_main_runtime_collector);

    runtime
        .on_exit_handlers
        .push(Arc::new(wutengine_graphics::persist_pipeline_cache));
}

pub(crate) trait MainRuntimeEvent: wutengine_event::Event {
    fn handle(self: Arc<Self>, runtime: &mut Runtime);
}

#[derive(Clone)]
pub(super) struct AddOnExitHandler(pub(super) Arc<dyn Fn() + Send + Sync>);

impl MainRuntimeEvent for AddOnExitHandler {
    fn handle(self: Arc<Self>, runtime: &mut Runtime) {
        runtime.on_exit_handlers.push(self.0.clone());
    }
}

#[derive(Clone)]
pub(super) struct AddOnExitRequestedHandler(pub(super) Arc<dyn Fn() -> bool + Send + Sync>);

impl MainRuntimeEvent for AddOnExitRequestedHandler {
    fn handle(self: Arc<Self>, runtime: &mut Runtime) {
        runtime.on_exit_requested_handlers.push(self.0.clone());
    }
}
