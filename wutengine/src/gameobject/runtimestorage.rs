//! [GameObject] storage for the WutEngine [crate::runtime::Runtime]

use std::collections::HashMap;

use super::{GameObject, GameObjectId};

/// A container for the runtime storage of WutEngine [GameObject] structs, and their related data
#[derive(Debug)]
pub(crate) struct GameObjectStorage {
    /// A map of [GameObjectId]s to indices into the [Self::objects] array
    pub(crate) identmap: HashMap<GameObjectId, usize>,

    /// The current [GameObject]s
    pub(crate) objects: Vec<GameObject>,
}

#[profiling::all_functions]
impl GameObjectStorage {
    /// Creates a new empty [GameObjectStorage]
    pub(crate) fn new() -> Self {
        Self {
            identmap: HashMap::default(),
            objects: Vec::new(),
        }
    }

    /// Adds the given set of [GameObject]s to the storage
    pub(crate) fn add_new_gameobjects(
        &mut self,
        gameobjects: impl IntoIterator<Item = GameObject>,
    ) {
        for new_gameobject in gameobjects.into_iter() {
            match self.identmap.contains_key(&new_gameobject.id) {
                true => log::error!(
                    "Tried to add an already existing GameObject, ignoring : {}",
                    new_gameobject.id
                ),
                false => {
                    let go_id = new_gameobject.id;
                    let new_idx = self.objects.len();

                    self.identmap.insert(go_id, new_idx);

                    log::debug!(
                        "Added new GameObject \"{}\" with ID {} at index {}",
                        new_gameobject.name,
                        go_id,
                        new_idx
                    );

                    self.objects.push(new_gameobject);
                }
            }
        }
    }
}
