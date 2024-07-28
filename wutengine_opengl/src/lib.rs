use core::ffi::CStr;
use std::collections::HashMap;

use window::Window;
use wutengine_graphics::{
    renderer::{RenderContext, Renderable, WutEngineRenderer},
    windowing::{HasDisplayHandle, HasWindowHandle, WindowIdentifier},
};

mod opengl {
    include!(concat!(env!("OUT_DIR"), "/gl_generated_bindings.rs"));
}

mod embedded {
    use include_dir::{include_dir, Dir};

    pub static SHADERS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/shaders");
}

mod gltypes;
mod shader;
mod shaderattribute;
mod shaderprogram;
mod vao;
mod vbo;
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
        log::debug!("Creating window {}", id);

        if self.windows.contains_key(id) {
            log::error!("Cannot create window {} because it already exists", id);
            return;
        }

        self.windows
            .insert(id.clone(), Window::new(window, phys_size));
    }

    fn destroy_window(&mut self, id: &WindowIdentifier) {
        log::debug!("Destroying window {}", id);

        if self.windows.remove(id).is_none() {
            log::error!("Removing window {} failed because it did not exist", id);
        }
    }

    fn size_changed(&mut self, id: &WindowIdentifier, phys_size: (u32, u32)) {
        log::debug!(
            "Handling size change to {}x{} for {}",
            phys_size.0,
            phys_size.1,
            id
        );

        if let Some(window) = self.windows.get_mut(id) {
            window.size_changed(phys_size);
        } else {
            log::error!("Window {} unknown, not doing resize", id);
        }
    }

    fn render(&mut self, render_context: RenderContext, objects: &[Renderable]) {
        log::trace!(
            "Rendering context {:#?} with {} objects",
            render_context,
            objects.len()
        );

        if let Some(window) = self.windows.get_mut(render_context.window) {
            window.render(render_context, objects);
        } else {
            log::error!(
                "Window {} unknown, not doing rendering",
                render_context.window
            );
        }
    }
}
