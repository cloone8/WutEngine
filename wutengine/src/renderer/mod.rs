//! The various WutEngine renderers and rendering functionality

use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::renderer::WutEngineRenderer;
use wutengine_graphics::renderer::{Renderable, Viewport};
use wutengine_graphics::shader::resolver::ShaderResolver;

pub(crate) mod queue;
pub(crate) mod shader_resolver;

#[cfg(feature = "opengl")]
pub use wutengine_opengl::OpenGLRenderer;

use crate::windowing::{HasDisplayHandle, HasWindowHandle};

/// The no-op headless renderer.
/// Ignores all rendering commands, and will
/// leave any opened windows with undefined content.
#[derive(Debug)]
pub struct HeadlessRenderer;

impl WutEngineRenderer for HeadlessRenderer {
    fn new_window(
        &mut self,
        _id: &WindowIdentifier,
        _window: &(impl HasDisplayHandle + HasWindowHandle),
        _phys_size: (u32, u32),
    ) {
    }

    fn size_changed(&mut self, _id: &WindowIdentifier, _phys_size: (u32, u32)) {}

    fn destroy_window(&mut self, _id: &WindowIdentifier) {}

    fn render(&mut self, _render_context: &Viewport, _objects: &[Renderable]) {}

    fn build<R: ShaderResolver>(_shaders: R) -> Self {
        Self
    }
}
