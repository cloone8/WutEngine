use gl_from_raw_window_handle::GlContext;
use glam::U64Vec2;
use wutengine_core::renderer::{renderable::Renderable, WindowHandles, WindowId};

use crate::{
    opengl::{self, Gl},
    GL_CONFIG,
};

use crate::opengl::types::*;

pub struct Window {
    id: WindowId,
    context: GlContext,
    bindings: Gl,
}

impl Window {
    pub unsafe fn init(id: WindowId, handles: WindowHandles, viewport: (u32, u32)) -> Self {
        let context =
            GlContext::create_from_handles(handles.window, handles.display, GL_CONFIG).unwrap();

        let bindings = opengl::Gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        bindings.Viewport(0, 0, viewport.0 as GLint, viewport.1 as GLint);

        Self {
            id,
            context,
            bindings,
        }
    }

    pub unsafe fn render(&mut self, objects: &[Renderable]) {
        self.context.make_current();

        let gl = self.bindings.clone();

        gl.ClearColor(0.2, 0.3, 0.3, 1.0);
        gl.Clear(opengl::COLOR_BUFFER_BIT);

        self.context.swap_buffers();
    }
}
