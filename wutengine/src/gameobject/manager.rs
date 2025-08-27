use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, RwLock};

use crate::gameobject::GameObject;

use super::GameObjectId;
use wutengine_util::hash::nohash_hasher::IntMap;
use wutengine_util::{GlobalManager, Queue};

#[derive(Debug)]
pub(crate) struct GameObjectManager {
    pub(crate) gameobjects: RwLock<IntMap<GameObjectId, GameObject>>,
    pub(super) gameobject_queue: Queue<GameObject>,
}

impl GameObjectManager {
    fn new() -> Self {
        Self {
            gameobjects: RwLock::new(HashMap::default()),
            gameobject_queue: Queue::default(),
        }
    }
}

pub(crate) static GAMEOBJECT_MANAGER: GlobalManager<GameObjectManager> = GlobalManager::new();

/// Initializes the gameobject manager
pub(crate) fn init() {
    GlobalManager::init(&GAMEOBJECT_MANAGER, GameObjectManager::new());
}
