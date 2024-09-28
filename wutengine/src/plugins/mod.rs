use crate::windowing::winit::event::{DeviceEvent, DeviceId, WindowEvent};
use wutengine_graphics::windowing::WindowIdentifier;

use crate::command::Command;
use crate::runtime::RuntimeInitializer;

/// Trait for WutEngine plugins.
/// These plugins are meant to be lower-level extensions to the engine runtime,
/// allowing for responses to raw events as well as injection
/// of custom systems and commands at key points in the engine lifecycle
pub trait WutEnginePlugin: 'static {
    /// Called once right after [RuntimeInitializer::run] is called
    fn on_build(&mut self, _initializer: &mut RuntimeInitializer) {}

    /// Called once when the runtime has just been built, and is starting
    fn on_start(&mut self, _commands: &mut Command) {}

    /// Called before starting each update tick
    fn pre_update(&mut self, _commands: &mut Command) {}

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
