//! Contains the data and supporting functionality required for each component to function

use super::Component;

/// Per-[Component] data
#[derive(Debug)]
pub(crate) struct ComponentData {
    pub(crate) component: Box<dyn Component>,
    pub(crate) state: ComponentState,
}

impl ComponentData {
    pub(crate) fn new(component: Box<dyn Component>) -> Self {
        ComponentData {
            component,
            state: ComponentState::ReadyForStart,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ComponentState {
    ReadyForStart,
    Active,
    Dying,
}
