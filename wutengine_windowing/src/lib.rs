//! Windowing and display management

use winit::window::WindowAttributes;
use wutengine_util::GlobalManager;

use crate::display::{DISPLAY_MANAGER, DisplayManager};
use crate::window::{
    WINDOW_MANAGER, WindowIdentifier, WindowManager, WindowResizedEvent, on_window_resized_event,
};

pub mod display;
pub mod window;

pub fn init(
    active_event_loop: &winit::event_loop::ActiveEventLoop,
    proxy: winit::event_loop::EventLoopProxy<WutEngineWinitEvent>,
) {
    GlobalManager::init(&DISPLAY_MANAGER, DisplayManager::new(active_event_loop));
    GlobalManager::init(&WINDOW_MANAGER, WindowManager::new(proxy));

    wutengine_event::subscribe::<WindowResizedEvent>(on_window_resized_event);
}

#[derive(Debug)]
pub enum WutEngineWinitEvent {
    CreateWindowRequested(WindowIdentifier, WindowAttributes),
}
