//! Contains the data and supporting functionality required for each component to function

use super::Component;

/// Per-[Component] data
#[derive(Debug)]
pub(crate) struct ComponentData {
    /// The actual [Component] implementation
    pub(crate) component: Box<dyn Component>,

    /// The current lifecycle state of the component
    pub(crate) state: ComponentState,
}

impl ComponentData {
    /// Creates a new [ComponentData] instance for the given
    /// component instance, with the [ComponentState::ReadyForStart]
    /// state.
    pub(crate) fn new(component: Box<dyn Component>) -> Self {
        ComponentData {
            component,
            state: ComponentState::ReadyForStart,
        }
    }

    pub(crate) fn get_inner_cast<T: Component>(&self) -> Option<&T> {
        let as_ref = self.component.as_ref().as_any();

        as_ref.downcast_ref::<T>()
    }

    pub(crate) fn get_inner_cast_mut<T: Component>(&mut self) -> Option<&mut T> {
        let as_ref = self.component.as_mut().as_any_mut();

        as_ref.downcast_mut::<T>()
    }
}

/// A component lifecycle state
#[derive(Debug, Clone, Copy)]
pub(crate) enum ComponentState {
    /// Component is ready for starting, but has
    /// not actually started yet
    ReadyForStart,

    /// Component is active and running
    Active,

    /// Component has been queued for cleanup, but its destroy callback
    /// still needs to be called
    Dying,
}
