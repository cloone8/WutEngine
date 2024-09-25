use wutengine_core::Component;
use wutengine_graphics::{color::Color, renderer::RenderContext, windowing::WindowIdentifier};

#[derive(Debug)]
pub struct Camera {
    pub display: WindowIdentifier,
    pub clear_color: Color,
}

impl Component for Camera {}

impl Camera {
    pub(crate) fn to_context(&self) -> RenderContext {
        RenderContext {
            window: self.display.clone(),
            clear_color: self.clear_color,
        }
    }
}
