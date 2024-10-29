use core::marker::PhantomData;
use std::sync::Mutex;

use crate::gameobject::GameObject;

/// The context used for interacting with engine-related APIs
#[must_use = "The commands within the context must be consumed"]
#[derive(Debug)]
pub struct EngineContext<'a> {
    new_gameobjects: Mutex<Vec<GameObject>>,
    ph: PhantomData<&'a ()>,
}

impl<'a> EngineContext<'a> {
    /// Construct a new, empty, [EngineContext]
    pub(crate) fn new() -> Self {
        Self {
            new_gameobjects: Mutex::new(Vec::new()),
            ph: PhantomData,
        }
    }

    /// Extracts the commands from this context
    pub(crate) fn consume(self) -> Vec<GameObject> {
        self.new_gameobjects.into_inner().unwrap()
    }

    /// Spawns the given GameObject within the world, as soon as possible. Note
    /// that this is _not_ instant.
    pub fn spawn_gameobject(&self, gameobject: GameObject) {
        let mut locked = self.new_gameobjects.lock().unwrap();

        locked.push(gameobject);
    }
}
