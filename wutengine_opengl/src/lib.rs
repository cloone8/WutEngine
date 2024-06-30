use gl_from_raw_window_handle::{GlConfig, GlContext, Profile};
use glam::U64Vec2;
use opengl::{types::*, Gl};
use window::Window;
use wutengine_core::{
    fastmap::FastMap,
    id::instance::InstanceID,
    renderer::{renderable::Renderable, WindowHandles, WindowId, WutEngineRenderer},
};

mod opengl;
mod window;

const GL_CONFIG: GlConfig = GlConfig {
    version: (4, 1),
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
};

pub struct OpenGLRenderer {
    windows: FastMap<Window>,
}

impl WutEngineRenderer for OpenGLRenderer {
    const NAME: &'static str = "OpenGL";

    fn init() -> Self {
        log::info!("Initializing OpenGL rendering backend");

        Self {
            windows: FastMap::new(),
        }
    }

    fn init_window(&mut self, id: WindowId, handles: WindowHandles, viewport: U64Vec2) {
        log::info!("Initializing OpenGL context for window with ID {}", id);

        self.windows
            .insert(unsafe { Window::init(id, handles, viewport) });
    }

    fn render(&mut self, window: WindowId, objects: &[Renderable]) {
        let window = self.windows.get_mut(window.id()).unwrap();

        unsafe { window.render(objects) };
    }
}
