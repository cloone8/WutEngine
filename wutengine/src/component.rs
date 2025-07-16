use core::any::Any;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use wutengine_util::hash::nohash_hasher::IntMap;
use wutengine_util::{GlobalManager, TypeName, unique_id_type};

use crate::gameobject::{self, GameObjectId};

#[derive(Debug)]
struct ComponentManager {
    components: RwLock<IntMap<ComponentId, ComponentData>>,
    queued: Mutex<IntMap<ComponentId, ComponentData>>,
}

impl ComponentManager {
    pub(crate) fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::default()),
            queued: Mutex::new(HashMap::default()),
        }
    }
}

static COMPONENT_MANAGER: GlobalManager<ComponentManager> = GlobalManager::new();

pub(crate) fn init() {
    GlobalManager::init(&COMPONENT_MANAGER, ComponentManager::new());
}

#[profiling::function]
pub(crate) fn handle_enable_disable() {
    log::trace!("Handling enabled/disabled Components");

    let components = COMPONENT_MANAGER.components.read().unwrap();

    for (_, component) in components.iter() {
        let context = ComponentContext {
            gameobject: component.gameobject,
            this: component.id,
        };

        let mut private_state = component.private.lock().unwrap();
        let requested_state = *component.requested_state.lock().unwrap();

        match (private_state.cur_state, requested_state) {
            (ComponentState::Destroyed, _) => {
                unreachable!("Component current state cannot be 'Destroyed'")
            }

            (ComponentState::Disabled, ComponentState::Enabled) => {
                if !private_state.started {
                    log::debug!("Starting Component {component}");
                    private_state.component.on_create(context);
                    private_state.started = true;
                }

                log::debug!("Enabling Component {component}");

                private_state.component.on_enable(context);
                private_state.cur_state = ComponentState::Enabled;
            }
            (ComponentState::Enabled, ComponentState::Disabled) => {
                log::debug!("Disabling Component {component}");
                private_state.component.on_disable(context);
                private_state.cur_state = ComponentState::Disabled;
            }

            _ => (), // Nothing to do
        }
    }
}

#[profiling::function]
pub(crate) fn handle_destruction() {
    log::trace!("Handling destroyed Components");

    let components = COMPONENT_MANAGER.components.read().unwrap();

    let mut to_delete = Vec::new();

    for (id, component) in components.iter() {
        let requested_state = *component.requested_state.lock().unwrap();

        if requested_state != ComponentState::Destroyed {
            continue;
        }

        log::debug!("Destroying Component {component}");

        let context = ComponentContext {
            gameobject: component.gameobject,
            this: component.id,
        };

        let mut private_state = component.private.lock().unwrap();

        let cur_state = private_state.cur_state;

        if cur_state == ComponentState::Enabled {
            private_state.component.on_disable(context);
        }

        private_state.component.on_destroy(context);

        gameobject::notify_component_destroyed(component.gameobject, *id);

        to_delete.push(*id);
    }

    core::mem::drop(components); // Exchange read lock to write lock

    let mut components = COMPONENT_MANAGER.components.write().unwrap();

    components.retain(|id, _| !to_delete.contains(id));
}

#[profiling::function]
pub(crate) fn add_component(component: Box<dyn Component>, owner: GameObjectId) -> ComponentId {
    let component_type_name = component.as_ref().type_name();

    log::trace!("Adding new Component of type {component_type_name} to GameObject {owner}",);

    let id = ComponentId::new();

    let mut components = COMPONENT_MANAGER.queued.lock().unwrap();

    components.insert(
        id,
        ComponentData {
            id,
            component_type_name,
            gameobject: owner,
            private: Mutex::new(ComponentPrivateData {
                started: false,
                cur_state: ComponentState::Disabled,
                component,
            }),
            requested_state: Mutex::new(ComponentState::Enabled),
        },
    );

    log::debug!("Added new Component {id} of type {component_type_name} to GameObject {owner}",);

    id
}

#[profiling::function]
pub(crate) fn add_queued() {
    log::trace!("Adding queued components to main runtime");

    let mut components = COMPONENT_MANAGER.components.write().unwrap();

    COMPONENT_MANAGER
        .queued
        .lock()
        .unwrap()
        .drain()
        .for_each(|(id, comp)| {
            components.insert(id, comp);
        });
}

#[profiling::function]
pub(crate) fn run_on_active_components(
    mut func: impl FnMut(&mut Box<dyn Component>, ComponentContext),
) {
    let components = COMPONENT_MANAGER.components.read().unwrap();

    for component in components.values() {
        let mut private = component.private.lock().unwrap();

        if private.cur_state != ComponentState::Enabled {
            continue;
        }

        func(
            &mut private.component,
            ComponentContext {
                gameobject: component.gameobject,
                this: component.id,
            },
        );
    }
}

#[profiling::function]
pub(crate) fn run_for<const INCLUDE_INACTIVE: bool, 'id>(
    ids: impl IntoIterator<Item = &'id ComponentId>,
    mut func: impl FnMut(&ComponentData),
) {
    let components = COMPONENT_MANAGER.components.read().unwrap();

    for id in ids.into_iter() {
        let component = components
            .get(id)
            .expect("Could not find expected component");

        if !INCLUDE_INACTIVE {
            let cur_state = component.private.lock().unwrap().cur_state;

            if cur_state != ComponentState::Enabled {
                continue;
            }
        }

        func(component);
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
    fn on_update(&mut self, _context: ComponentContext) {}
    fn on_render(&mut self, _context: ComponentContext) {}
    fn on_fixed_update(&mut self, _context: ComponentContext) {}
    fn on_disable(&mut self, _context: ComponentContext) {}
    fn on_destroy(&mut self, _context: ComponentContext) {}
}

pub fn destroy(id: ComponentId) {
    let components = COMPONENT_MANAGER.components.read().unwrap();

    let component = if let Some(comp) = components.get(&id) {
        comp
    } else {
        log::error!("Tried to destroy unknown component {id}");
        return;
    };

    component.request_state(ComponentState::Destroyed);
}

#[derive(Debug)]
pub(crate) struct ComponentData {
    id: ComponentId,
    component_type_name: &'static str,
    gameobject: GameObjectId,
    requested_state: Mutex<ComponentState>,
    private: Mutex<ComponentPrivateData>,
}

#[derive(Debug)]
struct ComponentPrivateData {
    started: bool,
    cur_state: ComponentState,
    component: Box<dyn Component>,
}

impl ComponentData {
    pub(crate) fn request_state(&self, state: ComponentState) {
        let mut requested_state = self.requested_state.lock().unwrap();

        if *requested_state == ComponentState::Destroyed {
            // Destruction is always permanent
            return;
        }

        *requested_state = state;
    }
}

impl core::fmt::Display for ComponentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let component_typename = self.component_type_name;

        write!(f, "{component_typename}(id={})", self.id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComponentState {
    /// Component disabled and not queued for activation
    Disabled,

    /// Component enabled. Normal state
    Enabled,

    /// Component queued for destruction. Final
    Destroyed,
}

impl ComponentState {
    pub(crate) const fn is_inactive(self) -> bool {
        !matches!(self, ComponentState::Enabled)
    }
}

unique_id_type! {
    /// The ID of a [Component]
    ComponentId
}
