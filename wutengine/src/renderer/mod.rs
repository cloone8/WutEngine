use wutengine_graphics::{
    renderer::WutEngineRenderer,
    windowing::{HasDisplayHandle, HasWindowHandle, WindowIdentifier},
};

#[cfg(feature = "opengl")]
pub use wutengine_opengl::OpenGLRenderer;

#[derive(Debug, Default)]
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
}