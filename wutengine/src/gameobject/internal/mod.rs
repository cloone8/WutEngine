//! Internal GameObject functionality

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, RwLock};

use runtimestorage::GameObjectStorage;

use super::GameObject;

mod runtimestorage;

pub(in crate::gameobject) static CREATION_QUEUE: Mutex<Vec<GameObject>> = Mutex::new(Vec::new());
pub(in crate::gameobject) static STORAGE: RwLock<MaybeUninit<GameObjectStorage>> =
    RwLock::new(MaybeUninit::uninit());

#[profiling::function]
pub(crate) fn init_storage() {
    static WAS_INIT: AtomicBool = AtomicBool::new(false);

    let was_init = WAS_INIT.swap(true, Ordering::SeqCst);

    assert!(!was_init, "GameObject storage already initialized");

    STORAGE.write().unwrap().write(GameObjectStorage::new());
}

#[profiling::function]
pub(crate) fn with_storage<F>(closure: F)
where
    F: FnOnce(&GameObjectStorage),
{
    let locked = STORAGE.read().unwrap();

    let storage = unsafe { locked.assume_init_ref() };

    closure(storage);
}

#[profiling::function]
pub(crate) fn with_storage_mut<F>(closure: F)
where
    F: FnOnce(&mut GameObjectStorage),
{
    let mut locked = STORAGE.write().unwrap();

    let storage = unsafe { locked.assume_init_mut() };

    closure(storage);
}

#[profiling::function]
pub(crate) fn take_creation_queue() -> Vec<GameObject> {
    let mut locked = CREATION_QUEUE.lock().unwrap();

    std::mem::take(&mut *locked)
}
