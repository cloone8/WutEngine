//! Functionality and definitions for the main [GameObject] type

use std::sync::RwLock;

use crate::component::Component;
use crate::component::data::{ComponentData, ComponentState};

pub(crate) mod internal;

wutengine_util_macro::generate_atomic_id! {

    /// The at-runtime ID of a GameObject
    GameObjectId
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
    pub(crate) components: RwLock<Vec<ComponentData>>,
}

#[profiling::all_functions]
impl GameObject {
    /// Creates a new [GameObject] that is not yet loaded into the world.
    pub fn new(name: Option<impl Into<String>>) -> Self {
        let name = name.map(|s| s.into()).unwrap_or("GameObject".to_string());

        Self {
            id: GameObjectId::new(),
            name,
            components: RwLock::new(Vec::new()),
        }
    }

    /// Adds a new component to this [GameObject]
    pub fn add_component(&mut self, component: impl Component) {
        self.components
            .get_mut()
            .unwrap()
            .push(ComponentData::new(Box::new(component)));
    }

    /// Removes all components that are currently marked as dying, without running
    /// their [crate::component::Component::on_destroy] callbacks. These should be run prior to calling this
    /// method through some other mechanism.
    pub(crate) fn remove_dying_components(&mut self) {
        log::trace!(
            "Removing dying components from object {} (ID {})",
            self.name,
            self.id
        );

        self.components
            .get_mut()
            .unwrap()
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
            .unwrap()
            .retain(|c| !matches!(c.state, ComponentState::ReadyForStart));
    }
}

/// Spawns the given [GameObject] as soon as possible
#[profiling::function]
pub fn spawn(go: GameObject) {
    internal::CREATION_QUEUE.lock().unwrap().push(go);
}
