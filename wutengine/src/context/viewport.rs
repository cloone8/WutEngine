use core::marker::PhantomData;
use std::sync::Mutex;

use wutengine_graphics::renderer::Viewport;

/// The viewport context, used for interacting with [Viewport] related APIs
#[must_use = "The commands within the context must be consumed"]
pub struct ViewportContext<'a> {
    viewports: Mutex<Vec<Viewport>>,
    ph: PhantomData<&'a ()>,
}

impl<'a> ViewportContext<'a> {
    /// Creates a new, empty, [ViewportContext]
    pub(crate) fn new() -> Self {
        ViewportContext {
            viewports: Mutex::new(Vec::new()),
            ph: PhantomData,
        }
    }

    /// Consumes the context, returning the list of [Viewport] instances that were added to it.
    pub(crate) fn consume(self) -> Vec<Viewport> {
        self.viewports.into_inner().unwrap()
    }

    /// Submits the command to render the given [Viewport] this frame.
    pub fn render_viewport(&self, viewport: Viewport) {
        let mut locked = self.viewports.lock().unwrap();
        locked.push(viewport);
    }
}
