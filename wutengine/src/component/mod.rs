//! Component related functionality and data

use core::any::Any;
use core::fmt::Debug;
use core::marker::PhantomData;

use crate::context::{
    EngineContext, GameObjectContext, GraphicsContext, MessageContext, ViewportContext,
};
use crate::context::{PluginContext, WindowContext};
use crate::runtime::messaging::Message;

pub(crate) mod data;

/// A component, the core programmable unit in WutEngine.
pub trait Component: Any + Send + Sync + Debug {
    /// Called before the first update cycle this component is active in
    fn on_start(&mut self, _context: &mut Context) {}

    /// Called right before this component is destroyed.
    fn on_destroy(&mut self, _context: &mut Context) {}

    /// The physics update hook. Any interaction with the physics
    /// components should happen here
    fn physics_update(&mut self, _context: &mut Context) {}

    /// Post-physics update hook. Used for any interactions
    /// following updates to physics components.
    fn post_physics_update(&mut self, _context: &mut Context) {}

    /// The pre-update hook. Runs before all the update hooks
    fn pre_update(&mut self, _context: &mut Context) {}

    /// The main update hook. Runs each frame. Use this in most cases
    fn update(&mut self, _context: &mut Context) {}

    /// The pre-render hook. Runs after the update phase. Use this for submitting
    /// rendering commands
    fn pre_render(&mut self, _context: &mut Context) {}

    /// Called for each message that might be relevant for this component.
    fn on_message(&mut self, _context: &mut Context, _message: &Message) {}

    /// Converts the component reference to a dyn [Any] reference.
    fn as_any(&self) -> &dyn Any;

    /// Converts the component mutable reference to a dyn [Any] mutable reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// The main context handed to each component each frame
pub struct Context<'a> {
    /// Engine related APIs and commands
    pub engine: &'a EngineContext,

    /// Information and APIs related to the gameobject this component is on
    pub gameobject: GameObjectContext<'a>,

    /// The message context
    pub message: &'a MessageContext<'a>,

    /// The loaded plugins
    pub plugin: &'a PluginContext<'a>,

    /// Information and APIs for interacting with viewports
    pub viewport: &'a ViewportContext,

    /// Graphics and rendering APIs
    pub graphics: &'a GraphicsContext,

    /// Window information and APIs
    pub window: &'a WindowContext<'a>,

    /// Engine APIs and functions for the current component
    pub this: ComponentContext<'a>,
}

/// Context for interacting with APIs and functions related to the
/// current component.
#[derive(Debug)]
pub struct ComponentContext<'a> {
    /// Whether the component should be marked as dying ASAP.
    pub(crate) should_die: bool,

    ph: PhantomData<&'a ()>,
}

impl ComponentContext<'_> {
    /// Creates a new, empty context
    pub(crate) fn new() -> Self {
        Self {
            should_die: false,
            ph: PhantomData,
        }
    }
    /// Marks the component this context is for as dying, preventing further new component
    /// hooks to be called on it and allowing it to be cleaned up by the engine runtime.
    pub fn destroy(&mut self) {
        self.should_die = true;
    }
}
