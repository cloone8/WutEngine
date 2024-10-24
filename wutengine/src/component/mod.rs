//! Component related functionality and data

use core::any::Any;

use crate::context::{EngineContext, GameObjectContext, GraphicsContext, ViewportContext};
use crate::context::{PluginContext, WindowContext};

/// A component, the core programmable unit in WutEngine.
pub trait Component: Any + Send + Sync {
    /// The pre-update hook. Runs before all the update hooks
    fn pre_update(&mut self, _context: &mut Context) {}

    /// The main update hook. Runs each frame. Use this in most cases
    fn update(&mut self, _context: &mut Context) {}

    /// The pre-render hook. Runs after the update phase. Use this for submitting
    /// rendering commands
    fn pre_render(&mut self, _context: &mut Context) {}

    /// Converts the component reference to a dyn [Any] reference.
    fn as_any(&self) -> &dyn Any;

    /// Converts the component mutable reference to a dyn [Any] mutable reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// The main context handed to each component each frame
pub struct Context<'a> {
    /// Engine related APIs and commands
    pub engine: &'a EngineContext<'a>,

    /// Information and APIs related to the gameobject this component is on
    pub gameobject: GameObjectContext<'a>,

    /// The loaded plugins
    pub plugin: &'a PluginContext<'a>,

    /// Information and APIs for interacting with viewports
    pub viewport: &'a ViewportContext<'a>,

    /// Graphics and rendering APIs
    pub graphics: &'a GraphicsContext<'a>,

    /// Window information and APIs
    pub window: &'a WindowContext<'a>,
}
