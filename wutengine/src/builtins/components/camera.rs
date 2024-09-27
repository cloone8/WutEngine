use glam::Mat4;
use wutengine_core::Component;
use wutengine_graphics::{color::Color, renderer::RenderContext, windowing::WindowIdentifier};

/// A camera that renders to a viewport in a window.
#[derive(Debug)]
pub struct Camera {
    /// The window to render to. Must match the identifier of an opened window in order
    /// for the camera to render anything.
    pub display: WindowIdentifier,

    /// The background color of the camera for any unset pixel
    pub clear_color: Color,
}

impl Component for Camera {}

impl Camera {
    /// Gets the [RenderContext] for this camera, for submission
    /// to a rendering backend
    pub(crate) fn to_context(&self) -> RenderContext {
        RenderContext {
            window: self.display.clone(),
            clear_color: self.clear_color,
            view_mat: Mat4::IDENTITY,
            projection_mat: Mat4::IDENTITY,
        }
    }
}
