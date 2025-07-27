use core::any::{Any, TypeId};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use wutengine_util::hash::nohash_hasher::IntMap;
use wutengine_util::{GlobalManager, TypeName, unique_id_type};

use crate::gameobject::{self, GameObjectId};

#[derive(Debug)]
struct ComponentManager {
    components: RwLock<IntMap<ComponentId, Arc<ComponentData>>>,
    queued: Mutex<IntMap<ComponentId, Arc<ComponentData>>>,
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

        let cur_state = component.cur_state.get();
        let requested_state = component.requested_state.get();

        match (cur_state, requested_state) {
            (ComponentState::Destroyed, _) => {
                unreachable!("Component current state cannot be 'Destroyed'")
            }

            (ComponentState::Disabled, ComponentState::Enabled) => {
                if !component.started.swap(true, Ordering::SeqCst) {
                    log::debug!("Starting Component {component}");
                    component.implementation.lock().unwrap().on_create(context);
                }

                log::debug!("Enabling Component {component}");

                if component
                    .cur_state
                    .0
                    .swap(ComponentState::Enabled as u8, Ordering::SeqCst)
                    == ComponentState::Disabled as u8
                {
                    component.implementation.lock().unwrap().on_enable(context);
                }
            }
            (ComponentState::Enabled, ComponentState::Disabled) => {
                log::debug!("Disabling Component {component}");

                if component
                    .cur_state
                    .0
                    .swap(ComponentState::Disabled as u8, Ordering::SeqCst)
                    == ComponentState::Enabled as u8
                {
                    component.implementation.lock().unwrap().on_disable(context);
                }
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
        if component.requested_state.get() != ComponentState::Destroyed {
            continue;
        }

        log::debug!("Destroying Component {component}");

        let context = ComponentContext {
            gameobject: component.gameobject,
            this: component.id,
        };

        let mut component_impl = component.implementation.lock().unwrap();

        if component.cur_state.get() == ComponentState::Enabled {
            component_impl.on_disable(context);
        }

        component_impl.on_destroy(context);

        gameobject::notify_component_destroyed(component.gameobject, *id);

        to_delete.push(*id);
    }

    core::mem::drop(components); // Exchange read lock to write lock

    let mut components = COMPONENT_MANAGER.components.write().unwrap();

    components.retain(|id, _| !to_delete.contains(id));
}

#[profiling::function]
pub(crate) fn add_component(
    component: Box<dyn Component>,
    component_type: TypeId,
    owner: GameObjectId,
) -> ComponentId {
    let component_type_name = component.as_ref().type_name();

    log::trace!("Adding new Component of type {component_type_name} to GameObject {owner}",);

    let id = ComponentId::new();

    let mut components = COMPONENT_MANAGER.queued.lock().unwrap();

    components.insert(
        id,
        Arc::new(ComponentData {
            id,
            component_type_name,
            component_type,
            gameobject: owner,
            started: AtomicBool::new(false),
            cur_state: AtomicComponentState::new(ComponentState::Disabled),
            implementation: Mutex::new(component),
            requested_state: AtomicComponentState::new(ComponentState::Enabled),
        }),
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
pub(crate) fn run_on_active_components(mut func: impl FnMut(&mut dyn Component, ComponentContext)) {
    let components = COMPONENT_MANAGER.components.read().unwrap();

    for component in components.values() {
        if component.cur_state.get() != ComponentState::Enabled {
            continue;
        }

        let mut locked_component = component.implementation.lock().unwrap();

        func(
            locked_component.as_mut(),
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
            if component.cur_state.get() != ComponentState::Enabled {
                continue;
            }
        }

        func(component);
    }
}

#[profiling::function]
pub(crate) fn find_components(
    filter: impl Fn(&ComponentData) -> bool,
    out: &mut Vec<Arc<ComponentData>>,
) {
    let components = COMPONENT_MANAGER.components.read().unwrap();

    for component in components.values() {
        if filter(component) {
            out.push(component.clone());
        }
    }
}

#[profiling::function]
pub(crate) fn get_component(id: &ComponentId) -> Option<Arc<ComponentData>> {
    COMPONENT_MANAGER
        .components
        .read()
        .unwrap()
        .get(id)
        .cloned()
}

#[profiling::function]
pub(crate) fn get_components_of_type<C: Component>(out: &mut Vec<Arc<ComponentData>>) {
    let components = COMPONENT_MANAGER.components.read().unwrap();

    for component in components.values() {
        if component.component_type == TypeId::of::<C>() {
            out.push(component.clone());
        }
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
}

pub trait Renderer: Component {
    fn render_color<'a>(
        &mut self,
        encoder: &mut wgpu::RenderPass<'a>,
        target_format: wgpu::TextureFormat,
    );
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
    pub(crate) id: ComponentId,
    pub(crate) component_type_name: &'static str,
    pub(crate) component_type: TypeId,
    pub(crate) gameobject: GameObjectId,
    pub(crate) started: AtomicBool,
    pub(crate) requested_state: AtomicComponentState,
    pub(crate) cur_state: AtomicComponentState,
    pub(crate) implementation: Mutex<Box<dyn Component>>,
}

impl ComponentData {
    pub(crate) fn request_state(&self, state: ComponentState) {
        //TODO: Can we relax the SeqCst ordering?
        self.requested_state
            .0
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
                if val == ComponentState::Destroyed as u8 {
                    Some(ComponentState::Destroyed as u8)
                } else {
                    Some(state as u8)
                }
            })
            .unwrap();
    }
}

impl core::fmt::Display for ComponentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let component_typename = self.component_type_name;

        write!(f, "{component_typename}(id={})", self.id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum ComponentState {
    /// Component disabled and not queued for activation
    Disabled = 0,

    /// Component enabled. Normal state
    Enabled,

    /// Component queued for destruction. Final
    Destroyed,
}

impl ComponentState {
    pub(crate) const fn is_inactive(self) -> bool {
        !matches!(self, ComponentState::Enabled)
    }

    pub(crate) const fn from_u8(val: u8) -> Self {
        const DISABLED: u8 = ComponentState::Disabled as u8;
        const ENABLED: u8 = ComponentState::Enabled as u8;
        const DESTROYED: u8 = ComponentState::Destroyed as u8;

        match val {
            DISABLED => ComponentState::Disabled,
            ENABLED => ComponentState::Enabled,
            DESTROYED => ComponentState::Destroyed,
            _ => panic!("Invalid component state given"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AtomicComponentState(AtomicU8);

impl AtomicComponentState {
    const fn new(state: ComponentState) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    #[inline]
    fn get(&self) -> ComponentState {
        //TODO: Can we relax the seqcst?
        let val = self.0.load(Ordering::SeqCst);

        ComponentState::from_u8(val)
    }
}

unique_id_type! {
    /// The ID of a [Component]
    ComponentId
}
