use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use wutengine_graphics::{
    renderer::{RenderContext, Renderable},
    windowing::{HasDisplayHandle, HasWindowHandle},
};

use crate::opengl::{self, Gl};

pub struct Window {
    context: GlContext,
    bindings: Gl,
}

impl Window {
    pub fn new(handles: impl HasDisplayHandle + HasWindowHandle, size: (u32, u32)) -> Self {
        let context = unsafe {
            GlContext::create(
                &handles,
                GlConfig {
                    version: (3, 3),
                    profile: Profile::Core,
                    red_bits: 8,
                    blue_bits: 8,
                    green_bits: 8,
                    alpha_bits: 8,
                    depth_bits: 24,
                    stencil_bits: 8,
                    samples: None,
                    srgb: true,
                    double_buffer: true,
                    vsync: false,
                },
            )
            .unwrap()
        };

        unsafe {
            context.make_current();
        }

        let bindings = Gl::load_with(|s| context.get_proc_address(s));

        unsafe { bindings.Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap()) };

        Self { context, bindings }
    }

    pub fn size_changed(&mut self, size: (u32, u32)) {
        unsafe {
            self.context.make_current();
            self.bindings
                .Viewport(0, 0, size.0.try_into().unwrap(), size.1.try_into().unwrap());
        };
    }

    pub fn render(&mut self, render_context: RenderContext, objects: &[Renderable]) {
        unsafe {
            self.context.make_current();
        }

        let gl = &self.bindings;

        let clear_color = render_context.clear_color;

        unsafe {
            gl.ClearColor(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            gl.Clear(opengl::COLOR_BUFFER_BIT);
        }

        self.context.swap_buffers();
    }
}
