use core::any::{Any, TypeId};
use std::sync::Mutex;

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use wutengine_util::{TypeName, unique_id_type64};

pub mod renderer;
mod state;

use crate::component::renderer::Renderer;
use crate::component::state::{ComponentState, ComponentTargetState};
use crate::gameobject::GameObjectId;

unique_id_type64! {
    /// The ID of a [Component]
    pub ComponentId
}

#[derive(Debug)]
pub(crate) struct ComponentData {
    inner_type: TypeId,
    state: ComponentState,
    target_state: Mutex<ComponentTargetState>,
    pub(crate) implementation: Mutex<Box<dyn Component>>,
}

impl ComponentData {
    pub(crate) const fn is_enabled(&self) -> bool {
        self.state.is_enabled()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComponentContext {
    pub gameobject: GameObjectId,
    pub this: ComponentId,
}

pub trait Component: Any + TypeName + Send + Sync + core::fmt::Debug {
    fn on_create(&mut self, _context: ComponentContext) {}
    fn on_enable(&mut self, _context: ComponentContext) {}
    fn on_fixed_update(&mut self, _context: ComponentContext) {}
    fn on_update(&mut self, _context: ComponentContext) {}
    fn on_disable(&mut self, _context: ComponentContext) {}
    fn on_destroy(&mut self, _context: ComponentContext) {}

    fn as_renderer(&mut self) -> Option<&mut dyn Renderer> {
        None
    }

    fn wanted_callbacks() -> ComponentCallbacks
    where
        Self: Sized;
}

bitflags! {
    /// The set of callbacks that an implementation of [Component] has implemented and wants to be called
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ComponentCallbacks: u8 {
        /// The [Component] wants [Component::on_create] to be called
        const CREATE =          0b00000001;
        /// The [Component] wants [Component::on_enable] to be called
        const ENABLE =          0b00000010;
        /// The [Component] wants [Component::on_fixed_update] to be called
        const FIXED_UPDATE =    0b00000100;
        /// The [Component] wants [Component::on_update] to be called
        const UPDATE =          0b00001000;
        /// The [Component] wants [Component::on_disable] to be called
        const DISABLE =         0b00010000;
        /// The [Component] wants [Component::on_destroy] to be called
        const DESTROY =         0b00100000;
    }
}
