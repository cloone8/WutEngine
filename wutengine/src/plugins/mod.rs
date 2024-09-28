use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use wutengine_graphics::windowing::WindowIdentifier;

use crate::command::Command;
use crate::runtime::RuntimeInitializer;

pub trait WutEnginePlugin: 'static {
    /// Called once right after [RuntimeInitializer::run] is called
    fn on_build(&mut self, _initializer: &mut RuntimeInitializer) {}

    /// Called once when the runtime has just been built, and is starting
    fn on_start(&mut self, _commands: &mut Command) {}

    /// Called once for each raw window event returned by the windowing system (currently [winit])
    fn on_window_event(
        &mut self,
        _window: &WindowIdentifier,
        _event: &WindowEvent,
        _commands: &mut Command,
    ) {
    }

    /// Called once for each raw device event returned by the windowing system (currently [winit])
    fn on_device_event(
        &mut self,
        _device: DeviceId,
        _event: &DeviceEvent,
        _commands: &mut Command,
    ) {
    }
}
