//! Functionality and definitions for the main [GameObject] type

use core::fmt::Display;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::component::Component;

static NEXT_INDEX: AtomicU64 = AtomicU64::new(0);

/// The at-runtime ID of a GameObject
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameObjectId(u64);

impl Display for GameObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// The GameObject. The main entity within the game world. Contains a set of components that
/// run the actual gameplay logic.
#[derive(Debug)]
pub struct GameObject {
    /// The unique identifier for this [GameObject]
    pub(crate) id: GameObjectId,

    /// The name of this [GameObject]
    pub(crate) name: String,

    /// The [Component] types active on this [GameObject]
    pub(crate) components: Vec<Box<dyn Component>>,
}

impl GameObject {
    /// Creates a new [GameObject] that is not yet loaded into the world.
    pub fn new(name: Option<String>) -> Self {
        let name = name.unwrap_or("GameObject".to_string());

        Self {
            id: GameObjectId(NEXT_INDEX.fetch_add(1, Ordering::Relaxed)),
            name,
            components: Vec::new(),
        }
    }

    /// Adds a new component to this [GameObject]
    pub fn add_component(&mut self, component: Box<dyn Component>) {
        self.components.push(component);
    }
}
