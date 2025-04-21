//! Functionality and definitions for the main [GameObject] type

use core::cell::RefCell;
use core::fmt::Display;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::component::Component;
use crate::component::data::{ComponentData, ComponentState};

pub(crate) mod runtimestorage;

static NEXT_INDEX: AtomicU64 = AtomicU64::new(0);

/// The at-runtime ID of a GameObject
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameObjectId(u64);

impl Display for GameObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#016x}", self.0)
    }
}

/// The GameObject. The main entity within the game world. Contains a set of components that
/// run the actual gameplay logic.
#[derive(Debug)]
pub struct GameObject {
    /// The unique identifier for this [GameObject]
    pub id: GameObjectId,

    /// The name of this [GameObject]
    pub name: String,

    /// The [Component] types active on this [GameObject]
    pub(crate) components: RefCell<Vec<ComponentData>>,
}

#[profiling::all_functions]
impl GameObject {
    /// Creates a new [GameObject] that is not yet loaded into the world.
    pub fn new(name: Option<impl Into<String>>) -> Self {
        let name = name.map(|s| s.into()).unwrap_or("GameObject".to_string());

        Self {
            id: GameObjectId(NEXT_INDEX.fetch_add(1, Ordering::Relaxed)),
            name,
            components: RefCell::new(Vec::new()),
        }
    }

    /// Adds a new component to this [GameObject]
    pub fn add_component(&mut self, component: impl Component) {
        self.components
            .get_mut()
            .push(ComponentData::new(Box::new(component)));
    }

    /// Removes all components that are currently marked as dying, without running
    /// their [Component::destroy] callbacks. These should be run prior to calling this
    /// method through some other mechanism.
    pub(crate) fn remove_dying_components(&mut self) {
        log::trace!(
            "Removing dying components from object {} (ID {})",
            self.name,
            self.id
        );

        self.components
            .get_mut()
            .retain(|c| !matches!(c.state, ComponentState::Dying));
    }

    /// Removes all components that are currently marked as [ComponentState::ReadyForStart] without
    /// actually running their startup callbacks.
    pub(crate) fn cancel_component_creation(&mut self) {
        log::trace!(
            "Cancelling all components that are queued for startup from object {} (ID {})",
            self.name,
            self.id
        );

        self.components
            .get_mut()
            .retain(|c| !matches!(c.state, ComponentState::ReadyForStart));
    }
}
