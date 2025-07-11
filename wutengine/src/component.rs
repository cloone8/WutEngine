use core::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;

use wutengine_util::nohash_hasher::IntMap;
use wutengine_util::{GlobalManager, TypeName, generate_atomic_id};

use crate::gameobject::GameObjectId;

#[derive(Debug)]
struct ComponentManager {
    components: RwLock<IntMap<ComponentId, ComponentData>>,
}

impl ComponentManager {
    pub(crate) fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::default()),
        }
    }
}

static COMPONENT_MANAGER: GlobalManager<ComponentManager> = GlobalManager::new();

pub(crate) fn init() {
    GlobalManager::init(&COMPONENT_MANAGER, ComponentManager::new());
}

#[profiling::function]
pub(crate) fn add_component(component: Box<dyn Component>, owner: GameObjectId) -> ComponentId {
    let component_type_name = component.as_ref().type_name();

    log::trace!("Adding new Component of type {component_type_name} to GameObject {owner}",);

    let id = ComponentId::new();

    let mut components = COMPONENT_MANAGER.components.write().unwrap();

    components.insert(
        id,
        ComponentData {
            id,
            gameobject: owner,
            state: ComponentState::Enabling,
            component,
        },
    );

    log::debug!("Added new Component {id} of type {component_type_name} to GameObject {owner}",);

    id
}

pub trait Component: Any + TypeName + Send + Sync + core::fmt::Debug {}

#[derive(Debug)]
pub(crate) struct ComponentData {
    id: ComponentId,
    gameobject: GameObjectId,
    state: ComponentState,
    component: Box<dyn Component>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComponentState {
    /// Component disabled and not queued for activation
    Disabled,

    /// Component disabled but queued for activation
    Enabling,

    /// Component enabled. Normal state
    Enabled,

    /// Component queued for deactivation
    Disabling,

    /// Component queued for destruction. Final
    Destroyed,
}

generate_atomic_id! {
    /// The ID of a [Component]
    ComponentId
}
