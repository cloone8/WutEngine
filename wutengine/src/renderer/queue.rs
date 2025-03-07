//! Internal render queue related functionality

use std::sync::Arc;

use glam::Mat4;
use wutengine_graphics::renderer::Viewport;

use crate::builtins::assets::{RawMaterial, RawMesh};
use crate::context::{GraphicsContext, ViewportContext};

/// The rendering queue for a given frame
pub(crate) struct RenderQueue {
    /// Viewports to render
    pub(crate) viewports: Vec<Viewport>,

    /// Objects to render
    pub(crate) renderables: Vec<RenderCommand>,
}

#[derive(Debug, Clone)]
pub(crate) struct RenderCommand {
    pub(crate) mesh: Arc<RawMesh>,
    pub(crate) material: Arc<RawMaterial>,
    pub(crate) object_to_world: Mat4,
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
