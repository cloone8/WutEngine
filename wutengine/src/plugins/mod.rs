//! Module for implementable engine plugins

use core::any::Any;
use core::fmt::Debug;
use std::collections::HashMap;

use crate::context::{MessageContext, WindowContext};
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
    /// Called once right after [RuntimeInitializer::run] is called
    fn on_build(&mut self, _initializer: &mut RuntimeInitializer) {}

    /// Called once when the runtime has just been built, and is starting
    fn on_start(&mut self, _context: &mut Context) {}

    /// The physics update hook. Any interaction with the physics
    /// components should happen here
    fn physics_update(&mut self, _context: &mut Context) {}

    /// Post-physics update hook. Used for any interactions
    /// following updates to physics components.
    fn post_physics_update(&mut self, _context: &mut Context) {}

    /// Hook exclusive to WutEngine plugins, ran after all physics and post-physics
    /// updates. At this point, all physics components have been synchronized, and can
    /// no longer be accessed by user scripts unless a message is sent to them.
    ///
    /// This basically allows the plugin to step any internal physics solvers,
    /// with the knowledge that all physics data that was going to be updated has
    /// actually been updated.
    fn physics_solver_update(&mut self, _context: &mut Context) {}

    /// Called before starting each update tick
    fn pre_update(&mut self, _context: &mut Context) {}

    /// Called on each update tick
    fn update(&mut self, _context: &mut Context) {}

    /// The pre-render hook. Runs after the update phase. Use this for submitting
    /// rendering commands
    fn pre_render(&mut self, _context: &mut Context) {}

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
    /// The message context
    pub message: MessageContext<'b>,

    /// The windowing context
    pub windows: WindowContext<'a>,
}

impl<'a, 'b, 'c> Context<'a, 'b, 'c> {
    /// Creates a new plugin context with the given parameters
    pub(crate) fn new(
        windows: &'a HashMap<WindowIdentifier, WindowData>,
        messages: &'b MessageQueue,
        objects: &'c GameObjectStorage,
    ) -> Self {
        Self {
            message: MessageContext::new(messages),
            windows: WindowContext::new(windows),
        }
    }
}
