//! Module for things surrounding user-defined global singletons

use core::any::{Any, TypeId};
use core::fmt::Debug;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc, RwLock};

#[cfg(debug_assertions)]
static GLOBAL_DATA_INITIALIZED: AtomicBool = AtomicBool::new(false);

static GLOBAL_DATA: RwLock<MaybeUninit<HashMap<TypeId, UntypedGlobal>>> =
    RwLock::new(MaybeUninit::uninit());

/// Initializes the global data store
pub(crate) fn init_globaldata() {
    log::trace!("Initializing global data");

    #[cfg(debug_assertions)]
    {
        let was_initialized = GLOBAL_DATA_INITIALIZED.swap(true, Ordering::Release);

        assert!(!was_initialized, "Global data already initialized!");
    }

    GLOBAL_DATA.write().unwrap().write(HashMap::default());
}

#[inline(always)]
#[track_caller]
fn assert_initialized() {
    #[cfg(debug_assertions)]
    {
        assert!(
            GLOBAL_DATA_INITIALIZED.load(Ordering::Acquire),
            "Global data not yet initialized"
        );
    }
}

/// Finds a [Global] of type `T`
pub fn find<T: Any + Send + Sync>() -> Option<Global<T>> {
    assert_initialized();

    let untyped = {
        let locked = GLOBAL_DATA.read().unwrap();
        let globals = unsafe { locked.assume_init_ref() };

        let found = globals.get(&TypeId::of::<T>())?;

        debug_assert!(!found.deleted.load(Ordering::Acquire));

        found.clone()
    };

    Some(untyped.into_typed::<T>().expect("Incorrectly typed global"))
}

/// Finds a [Global] of type `T`, and returns its cloned value
pub fn find_clone<T: Any + Send + Sync + Clone>() -> Option<T> {
    find::<T>().and_then(|f| f.get().cloned())
}

/// Creates a new global of the given type and value. If a global of the given type
/// already exists, returns an [Err] with `value`. Otherwise, returns [Ok]
pub fn create<T: Any + Send + Sync>(value: T) -> Result<(), T> {
    assert_initialized();

    let mut locked = GLOBAL_DATA.write().unwrap();
    let globals = unsafe { locked.assume_init_mut() };

    if let Entry::Vacant(e) = globals.entry(TypeId::of::<T>()) {
        e.insert(UntypedGlobal {
            deleted: Arc::new(AtomicBool::new(false)),
            data: Arc::new(value),
        });

        Ok(())
    } else {
        Err(value)
    }
}

/// Same as [create], but uses [Default] as the value
pub fn create_default<T: Any + Send + Sync + Default>() -> Result<(), T> {
    create(T::default())
}

/// Replaces the global of type `T` with a new value. This does _not_ modify the global value for
/// any other threads that currently have a reference to the global, but only marks it
/// as deleted. If the global does not exist, returns [Err]
pub fn replace<T: Any + Send + Sync>(value: T) -> Result<(), T> {
    assert_initialized();

    let mut locked = GLOBAL_DATA.write().unwrap();
    let globals = unsafe { locked.assume_init_mut() };

    if let Some(removed) = globals.remove(&TypeId::of::<T>()) {
        removed.deleted.store(true, Ordering::Release);

        globals.insert(
            TypeId::of::<T>(),
            UntypedGlobal {
                deleted: Arc::new(AtomicBool::new(false)),
                data: Arc::new(value),
            },
        );

        Ok(())
    } else {
        Err(value)
    }
}

/// Deletes the global with the given type from the WutEngine global storage.
/// This does prevents any instances of this global to be found with [find]
pub fn delete<T: Any + Send + Sync>() {
    assert_initialized();

    let mut locked = GLOBAL_DATA.write().unwrap();
    let globals = unsafe { locked.assume_init_mut() };

    let removed = globals.remove(&TypeId::of::<T>());

    if let Some(removed) = removed {
        debug_assert!(
            removed.clone().into_typed::<T>().is_ok(),
            "Wrong global type found"
        );

        let already_deleted = removed.deleted.swap(true, Ordering::AcqRel);

        debug_assert!(!already_deleted, "Global was already deleted");
    }
}

#[derive(Debug, Clone)]
struct UntypedGlobal {
    deleted: Arc<AtomicBool>,
    data: Arc<dyn Any + Send + Sync>,
}

impl UntypedGlobal {
    fn into_typed<T: Any + Send + Sync>(self) -> Result<Global<T>, Self> {
        match Arc::downcast::<T>(self.data) {
            Ok(downcast) => Ok(Global {
                deleted: self.deleted,
                data: downcast,
            }),
            Err(e) => Err(UntypedGlobal {
                deleted: self.deleted,
                data: e,
            }),
        }
    }
}

/// A WutEngine global value of type `T`.
/// In order to get a reference to the actual value,
/// use [Self::get].
#[derive(Debug, Clone)]
pub struct Global<T> {
    deleted: Arc<AtomicBool>,
    data: Arc<T>,
}

impl<T> Global<T> {
    /// Returns a reference to the underlying value, if it has not
    /// been deleted elsewhere.
    pub fn get(&self) -> Option<&T> {
        match self.deleted.load(Ordering::Acquire) {
            true => None,
            false => Some(&self.data),
        }
    }
}

impl<T> Global<T>
where
    T: Any + Send + Sync,
{
    /// Deletes this global and removes it from the WutEngine global storage.
    /// This also prevents other threads from accessing it, if they do not already have
    /// a reference
    pub fn delete(self) {
        delete::<T>();
    }
}
