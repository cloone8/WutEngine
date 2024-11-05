//! The main runtime and its main loop.

use std::collections::HashMap;

use messaging::MessageQueue;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;
use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::gameobject::{GameObject, GameObjectId};
use crate::plugins::WutEnginePlugin;
use crate::renderer::queue::RenderQueue;
use crate::windowing::window::Window;
use crate::windowing::WindowingEvent;

mod init;
mod main;
pub mod messaging;
mod threadpool;

pub use init::*;

/// The main runtime for WutEngine. Cannot be constructed directly. Instead,
/// construct a runtime with a [RuntimeInitializer]
/// TODO: Split up runtime object into multiple smaller structs
/// for cleaner code
pub struct Runtime<R: WutEngineRenderer> {
    identmap: HashMap<GameObjectId, usize>,
    objects: Vec<GameObject>,

    physics_update_interval: f32,
    physics_update_accumulator: f32,

    render_queue: RenderQueue,

    eventloop: EventLoopProxy<WindowingEvent>,

    window_id_map: HashMap<WindowId, WindowIdentifier>,
    windows: HashMap<WindowIdentifier, Window>,

    started: bool,

    plugins: Vec<Box<dyn WutEnginePlugin>>,
    renderer: R,
}
