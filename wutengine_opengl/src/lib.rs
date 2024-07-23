use std::collections::HashMap;

use window::Window;
use wutengine_core::{
    renderer::{HasDisplayHandle, HasWindowHandle, WutEngineRenderer},
    windowing::WindowIdentifier,
};

mod opengl {
    include!(concat!(env!("OUT_DIR"), "/gl_generated_bindings.rs"));
}

mod window;

#[derive(Default)]
pub struct OpenGLRenderer {
    windows: HashMap<WindowIdentifier, Window>,
}

impl WutEngineRenderer for OpenGLRenderer {
    fn new_window(
        &mut self,
        id: &WindowIdentifier,
        window: &(impl HasDisplayHandle + HasWindowHandle),
        phys_size: (u32, u32),
    ) {
        self.windows
            .insert(id.clone(), Window::new(window, phys_size));
    }

    fn destroy_window(&mut self, id: &WindowIdentifier) {
        self.windows.remove(id);
    }

    fn size_changed(&mut self, id: &WindowIdentifier, phys_size: (u32, u32)) {
        let window = self.windows.get_mut(id).unwrap();

        window.size_changed(phys_size);
    }

    fn render(
        &mut self,
        render_context: wutengine_core::renderer::RenderContext,
        objects: &[wutengine_core::renderer::Renderable],
    ) {
        let window = self.windows.get_mut(render_context.window).unwrap();

        window.render(render_context, objects);
    }
}
