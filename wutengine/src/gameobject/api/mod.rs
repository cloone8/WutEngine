use std::sync::{Arc, Mutex};

use arc_swap::ArcSwapOption;
use wutengine_util::hash::nohash_hasher::IntMap;

use crate::gameobject::GameObject;
use crate::gameobject::manager::GAMEOBJECT_MANAGER;
use crate::gameobject::state::{GameObjectState, GameObjectTargetState};
use crate::prelude::GameObjectId;

pub fn create(name: Option<String>) -> GameObjectId {
    let new_id = GameObjectId::new();

    let new_go = GameObject {
        id: new_id,
        name: ArcSwapOption::new(name.map(Arc::new)),
        state: GameObjectState::Disabled,
        target_state: Mutex::new(GameObjectTargetState::Enabled),
        components: IntMap::default(),
    };

    GAMEOBJECT_MANAGER.gameobject_queue.send(new_go);

    new_id
}
