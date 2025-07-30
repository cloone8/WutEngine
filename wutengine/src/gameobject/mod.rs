use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use wutengine_util::hash::nohash_hasher::IntMap;
use wutengine_util::{GlobalManager, unique_id_type64};

use crate::component::{self, Component, ComponentId};

#[derive(Debug)]
struct GameObjectManager {
    gameobjects: RwLock<IntMap<GameObjectId, GameObject>>,
}

impl GameObjectManager {
    fn new() -> Self {
        Self {
            gameobjects: RwLock::new(HashMap::default()),
        }
    }
}

static GAMEOBJECT_MANAGER: GlobalManager<GameObjectManager> = GlobalManager::new();

/// Initializes the gameobject manager
pub(crate) fn init() {
    GlobalManager::init(&GAMEOBJECT_MANAGER, GameObjectManager::new());
}

#[profiling::function]
pub(crate) fn handle_state_changes() {
    log::trace!("Handling GameObject state changes");

    let gameobjects = GAMEOBJECT_MANAGER.gameobjects.read().unwrap();

    for gameobject in gameobjects.values() {
        let mut private_data = gameobject.private.lock().unwrap();

        debug_assert_ne!(
            GameObjectState::Destroyed,
            private_data.cur_state,
            "{gameobject} is marked as destroyed in an active runtime"
        );

        let requested_state = *gameobject.requested_state.lock().unwrap();

        if private_data.cur_state == requested_state {
            // Nothing to do
            continue;
        }

        log::trace!(
            "{gameobject} transitioning from {} to {}",
            private_data.cur_state,
            requested_state
        );

        let requested_component_state = requested_state.to_component_state();
        component::run_for::<true>(&private_data.components, |component| {
            component.request_state(requested_component_state);
        });

        private_data.cur_state = requested_state;
    }
}

#[profiling::function]
pub(crate) fn cleanup_destroyed() {
    log::trace!("Cleaning up destroyed GameObjects");

    let mut gameobjects = GAMEOBJECT_MANAGER.gameobjects.write().unwrap();

    gameobjects.retain(|_id, gameobject| {
        let private_data = gameobject.private.lock().unwrap();

        if private_data.cur_state != GameObjectState::Destroyed {
            return true;
        }

        log::debug!("Destroying {gameobject}");

        assert!(
            private_data.components.is_empty(),
            "Destroying GameObject while it still has components attached"
        );

        false
    });
}

#[profiling::function]
pub fn create_object(name: Option<String>) -> GameObjectId {
    log::trace!("Creating new GameObject");

    let mut gameobjects = GAMEOBJECT_MANAGER.gameobjects.write().unwrap();
    let id = GameObjectId::new();

    gameobjects.insert(
        id,
        GameObject {
            id,
            name: Mutex::new(name),
            requested_state: Mutex::new(GameObjectState::Enabled),
            private: Mutex::new(GameObjectPrivate {
                cur_state: GameObjectState::Disabled,
                components: Vec::new(),
            }),
        },
    );

    log::debug!("Created GameObject with ID {id}");

    id
}

#[profiling::function]
pub fn add_component<C: Component>(gameobject: GameObjectId, component: C) {
    log::debug!("Adding Component to GameObject {gameobject}");

    let gameobjects = GAMEOBJECT_MANAGER.gameobjects.read().unwrap();

    let go = if let Some(go) = gameobjects.get(&gameobject) {
        go
    } else {
        log::error!("Cannot add component to unknown gameobject {gameobject}");
        return;
    };

    let component_id = crate::component::add_component(
        Box::new(component),
        core::any::TypeId::of::<C>(),
        gameobject,
    );

    let mut private_data = go.private.lock().unwrap();
    private_data.components.push(component_id);
}

pub fn destroy(id: GameObjectId) {
    let gameobjects = GAMEOBJECT_MANAGER.gameobjects.read().unwrap();

    let go = if let Some(go) = gameobjects.get(&id) {
        go
    } else {
        log::error!("Tried to destroy unknown GameObject {id}");
        return;
    };

    go.request_state(GameObjectState::Destroyed);
}

#[profiling::function]
pub(crate) fn notify_component_destroyed(owner: GameObjectId, component: ComponentId) {
    let gameobjects = GAMEOBJECT_MANAGER.gameobjects.read().unwrap();

    let gameobject = gameobjects
        .get(&owner)
        .expect("Attempted to notify non-existing owner of component destruction");

    let mut private_data = gameobject.private.lock().unwrap();

    let to_remove = private_data
        .components
        .iter()
        .position(|comp| *comp == component);

    private_data
        .components
        .swap_remove(to_remove.expect("Component to remove was not found on the GameObject"));
}

#[derive(Debug)]
pub struct GameObject {
    id: GameObjectId,
    name: Mutex<Option<String>>,
    requested_state: Mutex<GameObjectState>,
    private: Mutex<GameObjectPrivate>,
}

impl GameObject {
    pub(crate) fn request_state(&self, state: GameObjectState) {
        let mut requested_state = self.requested_state.lock().unwrap();

        if *requested_state == GameObjectState::Destroyed {
            // Destruction is always permanent
            return;
        }

        *requested_state = state;
    }
}

#[derive(Debug)]
struct GameObjectPrivate {
    cur_state: GameObjectState,
    components: Vec<ComponentId>,
}

impl core::fmt::Display for GameObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &*self.name.lock().unwrap() {
            Some(name) => write!(f, "GameObject({name})"),
            None => write!(f, "GameObject(id={})", self.id),
        }
    }
}

impl GameObject {
    #[inline(always)]
    pub fn create(name: Option<impl ToString>) -> GameObjectId {
        create_object(name.map(|n| n.to_string()))
    }

    #[inline(always)]
    pub fn add_component<C: Component>(id: GameObjectId, component: C) {
        add_component(id, component);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameObjectState {
    /// Object disabled
    Disabled,

    /// Object enabled. Normal state
    Enabled,

    /// Object queued for destruction. Final
    Destroyed,
}

impl core::fmt::Display for GameObjectState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GameObjectState::Disabled => write!(f, "Disabled"),
            GameObjectState::Enabled => write!(f, "Enabled"),
            GameObjectState::Destroyed => write!(f, "Destroyed"),
        }
    }
}

impl GameObjectState {
    const fn to_component_state(self) -> component::ComponentState {
        match self {
            GameObjectState::Disabled => component::ComponentState::Disabled,
            GameObjectState::Enabled => component::ComponentState::Enabled,
            GameObjectState::Destroyed => component::ComponentState::Destroyed,
        }
    }
}

unique_id_type64! {
    /// The ID of a [GameObject]
    pub GameObjectId
}

impl GameObjectId {
    #[inline(always)]
    pub fn add_component<C: Component>(self, component: C) {
        add_component(self, component)
    }
}
