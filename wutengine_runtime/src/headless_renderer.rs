use glam::U64Vec2;
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

    fn init_window(&mut self, id: WindowId, handles: WindowHandles, viewport: U64Vec2) {}

    fn render(&mut self, window: WindowId, objects: &[Renderable]) {}
}
