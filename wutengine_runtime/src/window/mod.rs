use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::WindowAttributes};
use wutengine_core::renderer::{WindowHandles, WutEngineRenderer};

use crate::{serialization::format::SerializationFormat, to_wutengine_window_id, WutEngine};

pub struct Window;

impl Window {
    pub fn open(
        engine: &mut WutEngine<impl WutEngineRenderer, impl SerializationFormat>,
        event_loop: &ActiveEventLoop,
        size: (u32, u32),
    ) {
        let attrs = WindowAttributes::default()
            .with_title("WutEngine")
            .with_inner_size(PhysicalSize {
                width: size.0,
                height: size.1,
            });

        let new_window = event_loop.create_window(attrs).unwrap();

        engine.renderer.init_window(
            to_wutengine_window_id(new_window.id()),
            WindowHandles::from_window(&new_window).unwrap(),
            size,
        );

        engine.windows.push(new_window);
    }
}
