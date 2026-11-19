//! Windowing API

use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::TrySendError;

use winit::event_loop::ActiveEventLoop;
use winit::window::Window as WinitWindow;
use winit::window::WindowAttributes;
use wutengine_macro::unique_id_type32;
use wutengine_sync::Mutex;

use crate::log;

unique_id_type32! {
    /// Handle to a WutEngine window
    pub Window
}

#[derive(Debug)]
struct WindowManager {
    queue_send: SyncSender<WindowRequest>,
    queue_recv: Mutex<Receiver<WindowRequest>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowRequest {
    Create(Window),
}

static WINDOW_MANAGER: OnceLock<WindowManager> = OnceLock::new();

/// Returns the [`WindowManager`]
///
/// Panics if not yet initialized
fn window_manager() -> &'static WindowManager {
    WINDOW_MANAGER
        .get()
        .expect("Window manager not yet initialized")
}

/// Initializes the windowing subsystem
pub(crate) fn init() {
    profiling::function_scope!();

    log::debug!("Initializing window manager");

    let (queue_send, queue_recv) = mpsc::sync_channel(32);

    WINDOW_MANAGER
        .set(WindowManager {
            queue_send,
            queue_recv: Mutex::new(queue_recv),
        })
        .expect("Window manager already initialized");
}

/// Sends a request to create a new window. The handle is returned immediately,
/// and the window itself is created as soon as possible on the main thread
#[must_use]
pub fn create() -> Window {
    let handle = Window::new();

    match window_manager()
        .queue_send
        .try_send(WindowRequest::Create(handle))
    {
        Ok(()) => handle,
        Err(TrySendError::Full(_)) => {
            log::error!(
                "Failed to create window because there are too many window creation requests in the queue"
            );
            handle // TODO: Return error? Might be too complex of an API for this edge case
        }
        Err(TrySendError::Disconnected(_)) => {
            panic!("Window manager dead");
        }
    }
}

pub(crate) fn run_queued(event_loop: &dyn ActiveEventLoop) {
    profiling::function_scope!();

    let queue = window_manager().queue_recv.lock();

    for request in queue.try_iter() {
        profiling::scope!("Handle request");

        log::trace!("Handling window request: {request:#?}");

        match request {
            WindowRequest::Create(window) => {
                let native_window = create_window(event_loop);
            }
        }
    }
}

fn create_window(event_loop: &dyn ActiveEventLoop) -> Arc<dyn WinitWindow> {
    let window = event_loop
        .create_window(WindowAttributes::default())
        .expect("Failed to create native window");

    Arc::from(window)
}
