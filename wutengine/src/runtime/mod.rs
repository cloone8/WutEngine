//! The main runtime and its main loop.

use core::sync::atomic::AtomicBool;
use std::collections::HashMap;

use messaging::MessageQueue;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;
use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::gameobject::runtimestorage::GameObjectStorage;
use crate::plugins::WutEnginePlugin;
use crate::windowing::WindowingEvent;
use crate::windowing::window::Window;

mod init;
mod main;
pub mod messaging;
mod threadpool;

pub use init::*;

/// Whether the runtime was requested to stop by the user through [global_fns::exit]
pub(crate) static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// The main runtime for WutEngine. Cannot be constructed directly. Instead,
/// construct a runtime with a [RuntimeInitializer]
/// TODO: Split up runtime object into multiple smaller structs
/// for cleaner code
pub struct Runtime<R: WutEngineRenderer> {
    obj_storage: GameObjectStorage,

    physics_update_interval: f32,
    physics_update_accumulator: f32,

    eventloop: EventLoopProxy<WindowingEvent>,

    window_id_map: HashMap<WindowId, WindowIdentifier>,
    windows: HashMap<WindowIdentifier, Window>,

    started: bool,

    plugins: Vec<Box<dyn WutEnginePlugin>>,
    renderer: R,
}

/// Stops the WutEngine runtime cleanly before the next frame. Can still finish the current frame
pub fn exit() {
    EXIT_REQUESTED.store(true, core::sync::atomic::Ordering::SeqCst);
}
