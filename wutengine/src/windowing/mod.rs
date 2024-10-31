//! Windowing and windows

use display::Display;
use winit::monitor::VideoModeHandle;
use winit::window::Fullscreen;

pub mod display;
pub mod window;

#[doc(inline)]
pub use wutengine_core::identifiers::WindowIdentifier;

#[doc(inline)]
pub use wutengine_graphics::renderer::HasDisplayHandle;

#[doc(inline)]
pub use wutengine_graphics::renderer::HasWindowHandle;

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
    BorderlessFullscreen(Display),

    /// Exclusive fullscreen
    ExclusiveFullscreen(Display, VideoModeHandle),
}

impl From<FullscreenType> for Option<Fullscreen> {
    fn from(value: FullscreenType) -> Self {
        //TODO: Switch this `From` function to `TryFrom`
        match value {
            FullscreenType::Windowed => None,
            FullscreenType::BorderlessFullscreen(display) => {
                Some(Fullscreen::Borderless(Some(display.handle)))
            }
            FullscreenType::ExclusiveFullscreen(display, mode) => {
                if mode.monitor() != display.handle {
                    log::error!("Could not set window to fullscreen, because the provided mode does not match the display");
                    None
                } else {
                    Some(Fullscreen::Exclusive(mode))
                }
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
