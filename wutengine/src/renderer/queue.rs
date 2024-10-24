//! Internal render queue related functionality

use wutengine_graphics::renderer::{Renderable, Viewport};

use crate::context::{GraphicsContext, ViewportContext};

/// The rendering queue for a given frame
pub(crate) struct RenderQueue {
    /// Viewports to render
    pub(crate) viewports: Vec<Viewport>,

    /// Objects to render
    pub(crate) renderables: Vec<Renderable>,
}

impl RenderQueue {
    /// Creates a new, empty, [RenderQueue]
    pub(crate) fn new() -> Self {
        Self {
            viewports: Vec::new(),
            renderables: Vec::new(),
        }
    }

    /// Consumes the given viewport context and adds its viewports to the queue
    pub(crate) fn add_viewports(&mut self, from_context: ViewportContext) {
        self.viewports.extend(from_context.consume());
    }

    /// Consumes the given graphics context and adds its renderables to the queue
    pub(crate) fn add_renderables(&mut self, from_context: GraphicsContext) {
        self.renderables.extend(from_context.consume());
    }
}
