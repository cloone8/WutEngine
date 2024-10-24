//! Windowing and windows

pub use winit;
use winit::window::Fullscreen;
use wutengine_graphics::windowing::WindowIdentifier;

/// The parameters that define a new window
#[derive(Debug)]
pub struct OpenWindowParams {
    /// The identifier for the new window
    pub id: WindowIdentifier,

    /// The title used for the window title bar
    pub title: String,

    /// The fullscreen/windowed mode of the new window
    pub mode: FullscreenType,

    /// Whether any existing windows with the same identifier will be replaced by this new window
    pub ignore_existing: bool,
}

/// The layout mode of a window
#[derive(Debug)]
pub enum FullscreenType {
    /// Windowed
    Windowed,

    /// Borderless fullscreen
    BorderlessFullscreenWindow,

    /// Exclusive fullscreen
    ExclusiveFullscreen,
}

impl From<FullscreenType> for Option<Fullscreen> {
    fn from(value: FullscreenType) -> Self {
        match value {
            FullscreenType::Windowed => None,
            FullscreenType::BorderlessFullscreenWindow => {
                todo!("Borderless fullscreen not yet implemented")
            }
            FullscreenType::ExclusiveFullscreen => {
                todo!("Exclusive fullscreen not yet implemented")
            }
        }
    }
}

/// An event that can be sent to [winit]
#[derive(Debug)]
pub(crate) enum WindowingEvent {
    /// A window should be opened with the given parameters
    OpenWindow(OpenWindowParams),
}
