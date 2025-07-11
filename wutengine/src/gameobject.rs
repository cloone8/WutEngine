use core::any::Any;
use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use wutengine_util::nohash_hasher::IntMap;
use wutengine_util::{GlobalManager, generate_atomic_id};

use crate::component::{Component, ComponentId};

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

pub(crate) fn init() {
    GlobalManager::init(&GAMEOBJECT_MANAGER, GameObjectManager::new());
}

pub fn create_object() -> GameObjectId {
    log::trace!("Creating new GameObject");

    let mut gameobjects = GAMEOBJECT_MANAGER.gameobjects.write().unwrap();
    let id = GameObjectId::new();

    gameobjects.insert(
        id,
        GameObject {
            id,
            state: GameObjectState::Enabling,
            components: RwLock::new(Vec::new()),
        },
    );

    log::debug!("Created GameObject with ID {id}");

    id
}

pub fn add_component(gameobject: GameObjectId, component: Box<dyn Component>) {
    log::debug!("Adding Component to GameObject {}", gameobject);

    let gameobjects = GAMEOBJECT_MANAGER.gameobjects.read().unwrap();

    let go = if let Some(go) = gameobjects.get(&gameobject) {
        go
    } else {
        log::error!("Cannot add component to unknown gameobject {gameobject}");
        return;
    };

    let component_id = crate::component::add_component(component, gameobject);

    go.components.write().unwrap().push(component_id);
}

#[derive(Debug)]
pub struct GameObject {
    id: GameObjectId,
    state: GameObjectState,
    components: RwLock<Vec<ComponentId>>,
}

impl GameObject {
    #[inline(always)]
    pub fn create() -> GameObjectId {
        create_object()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameObjectState {
    /// Object disabled and not queued for activation
    Disabled,

    /// Object disabled but queued for activation
    Enabling,

    /// Object enabled. Normal state
    Enabled,

    /// Object queued for deactivation
    Disabling,

    /// Object queued for destruction. Final
    Destroyed,
}

generate_atomic_id! {
    /// The ID of a [GameObject]
    GameObjectId
}
