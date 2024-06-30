use wutengine_core::renderer::{
    renderable::Renderable, WindowHandles, WindowId, WutEngineRenderer,
};

#[derive(Debug, Default)]
pub struct HeadlessRenderer;

impl WutEngineRenderer for HeadlessRenderer {
    const NAME: &'static str = "Headless";

    fn init() -> Self {
        log::info!("Initialized headless backend");
        Self
    }

    fn init_window(&mut self, _id: WindowId, _handles: WindowHandles, _viewport: (u32, u32)) {}

    fn render(&mut self, _window: WindowId, _objects: &[Renderable]) {}
}
