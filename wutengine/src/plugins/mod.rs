//! Module for implementable engine plugins

use core::any::Any;
use core::fmt::Debug;
use std::collections::HashMap;

use crate::context::{
    EngineContext, GraphicsContext, MessageContext, ViewportContext, WindowContext,
};
use crate::runtime::messaging::MessageQueue;
use crate::windowing::window::WindowData;
use crate::winit::event::{DeviceEvent, DeviceId, WindowEvent};
use wutengine_core::identifiers::WindowIdentifier;

use crate::runtime::RuntimeInitializer;

/// Trait for WutEngine plugins.
/// These plugins are meant to be lower-level extensions to the engine runtime,
/// allowing for responses to raw events as well as injection
/// of custom systems and commands at key points in the engine lifecycle
pub trait WutEnginePlugin: Any + Send + Sync + Debug {
    /// Casts the plugin to an instance of [Any], for possible downcasting later
    fn as_any(&self) -> &dyn Any;

    /// Called once right after [RuntimeInitializer::run] is called
    fn on_build(&mut self, _initializer: &mut RuntimeInitializer) {}

    /// Called once when the runtime has just been built, and is starting
    fn on_start(&mut self, _context: &mut Context) {}

    /// Called before starting each update tick
    fn pre_update(&mut self, _context: &mut Context) {}

    /// Called once for each raw window event returned by the windowing system (currently [winit])
    fn on_window_event(
        &mut self,
        _window: &WindowIdentifier,
        _event: &WindowEvent,
        _context: &mut Context,
    ) {
    }

    /// Called once for each raw device event returned by the windowing system (currently [winit])
    fn on_device_event(&mut self, _device: DeviceId, _event: &DeviceEvent, _context: &mut Context) {
    }
}

/// The context handed to most plugin hooks. Can be used to access the engine APIs
pub struct Context<'a> {
    /// The engine context
    pub engine: EngineContext<'a>,

    /// The message context
    pub message: MessageContext<'a>,

    /// The viewport context
    pub viewport: ViewportContext<'a>,

    /// The graphics context
    pub graphics: GraphicsContext<'a>,

    /// The windowing context
    pub windows: WindowContext<'a>,
}

impl<'a> Context<'a> {
    /// Creates a new plugin context with the given parameters
    pub(crate) fn new(
        windows: &'a HashMap<WindowIdentifier, WindowData>,
        messages: &'a MessageQueue,
    ) -> Self {
        Self {
            engine: EngineContext::new(),
            message: MessageContext::new(messages),
            viewport: ViewportContext::new(),
            graphics: GraphicsContext::new(),
            windows: WindowContext::new(windows),
        }
    }
}
