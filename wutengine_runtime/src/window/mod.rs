use winit::{dpi::PhysicalSize, event_loop::ActiveEventLoop, window::WindowAttributes};
use wutengine_core::renderer::{WindowHandles, WindowId, WutEngineRenderer};

use crate::{serialization::format::SerializationFormat, to_wutengine_window_id, WutEngine};

pub const MAIN_WINDOW_ID: &str = "MAIN";

#[derive(Debug)]
pub struct Window {
    /// The raw ID used in the engine backend
    pub id: WindowId,

    /// The human-readable ID used in application code to bind cameras to windows
    identifier: &'static str,

    /// The raw window
    raw: winit::window::Window,
}

impl Window {
    pub fn open(
        identifier: &'static str,
        engine: &mut WutEngine<impl WutEngineRenderer, impl SerializationFormat>,
        event_loop: &ActiveEventLoop,
        size: (u32, u32),
    ) -> WindowId {
        //TODO: Proper error handling

        assert!(
            engine
                .windows
                .iter()
                .filter(|window| window.identifier == identifier)
                .count()
                == 0
        );

        let attrs = WindowAttributes::default()
            .with_title("WutEngine")
            .with_inner_size(PhysicalSize {
                width: size.0,
                height: size.1,
            });

        let new_window = event_loop.create_window(attrs).unwrap();
        let wutengine_window_id = to_wutengine_window_id(new_window.id());

        engine.renderer.init_window(
            wutengine_window_id,
            WindowHandles::from_window(&new_window).unwrap(),
            size,
        );

        engine.windows.push(Self {
            id: wutengine_window_id,
            identifier,
            raw: new_window,
        });

        wutengine_window_id
    }
}
