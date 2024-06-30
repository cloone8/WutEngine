use wutengine_core::renderer::{Renderable, WutEngineRenderer};

#[derive(Debug, Default)]
pub struct HeadlessRenderer;

impl WutEngineRenderer for HeadlessRenderer {
    const NAME: &'static str = "Headless";

    fn init(&mut self) {
        log::info!("Initialized headless backend")
    }

    fn render(&mut self, objects: &[Renderable]) {
        log::info!("Rendering engine state");
    }
}
