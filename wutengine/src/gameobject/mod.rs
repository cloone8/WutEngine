use std::sync::{Arc, Mutex};

use arc_swap::ArcSwapOption;
use wutengine_util::hash::nohash_hasher::IntMap;
use wutengine_util::unique_id_type64;

use crate::component::{ComponentData, ComponentId};
use crate::gameobject::manager::GAMEOBJECT_MANAGER;
use crate::gameobject::state::{GameObjectState, GameObjectTargetState};

pub(crate) mod manager;
mod state;

mod api;

pub use api::*;

#[derive(Debug)]
pub(crate) struct GameObject {
    pub(crate) id: GameObjectId,
    name: ArcSwapOption<String>,
    state: GameObjectState,
    target_state: Mutex<GameObjectTargetState>,
    pub(crate) components: IntMap<ComponentId, ComponentData>,
}

impl GameObject {
    fn request_state(&self, state: GameObjectState) {
        let mut requested_state = self.target_state.lock().unwrap();

        if *requested_state == GameObjectTargetState::Destroyed {
            // Destruction is always permanent
            return;
        }

        *requested_state = state.into();
    }

    pub(crate) const fn is_enabled(&self) -> bool {
        self.state.is_enabled()
    }
}

impl core::fmt::Display for GameObject {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &*self.name.load() {
            Some(name) => write!(f, "GameObject({name})"),
            None => write!(f, "GameObject(id={})", self.id),
        }
    }
}

unique_id_type64! {
    /// The ID of a [GameObject]
    pub GameObjectId
}
