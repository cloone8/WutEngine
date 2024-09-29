use wutengine_graphics::{
    renderer::WutEngineRenderer,
    windowing::{HasDisplayHandle, HasWindowHandle, WindowIdentifier},
};

pub(crate) mod shader_resolver;

#[cfg(feature = "opengl")]
pub use wutengine_opengl::OpenGLRenderer;

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

    fn render(
        &mut self,
        _render_context: wutengine_graphics::renderer::RenderContext,
        _objects: &[wutengine_graphics::renderer::Renderable],
    ) {
    }

    fn build<R: wutengine_graphics::shader::resolver::ShaderResolver>(_shaders: R) -> Self {
        Self
    }
}
