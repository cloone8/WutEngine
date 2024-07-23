pub use raw_window_handle::HasDisplayHandle;
pub use raw_window_handle::HasWindowHandle;

use crate::windowing::WindowIdentifier;

pub struct RenderContext<'a> {
    pub window: &'a WindowIdentifier,
}

pub struct Renderable;

pub trait WutEngineRenderer: Default {
    fn new_window(
        &mut self,
        id: &WindowIdentifier,
        window: &(impl HasDisplayHandle + HasWindowHandle),
        phys_size: (u32, u32),
    );

    fn destroy_window(&mut self, id: &WindowIdentifier);

    fn size_changed(&mut self, id: &WindowIdentifier, phys_size: (u32, u32));

    fn render(&mut self, render_context: RenderContext, objects: &[Renderable]);
}
