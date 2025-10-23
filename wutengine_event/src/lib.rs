//! WutEngine event handling and dispatching
//!
//! Provides a global crate where any other WutEngine sub-crate (and by extension, WutEngine users) can hook into and publish new events.

use core::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::sync::RwLock;

use wutengine_util::GlobalManager;

/// A type that can be sent as an event through the WutEngine event system, and
/// can also be listened for
pub trait WutEngineEvent: Any + Send + Sync {}

/// Publishes the event, notifying all subscribers
#[profiling::function]
pub fn publish<T: WutEngineEvent>(event: T) {
    log::debug!("Sending event of type {}", type_name::<T>());

    let callbacks = EVENT_MANAGER.callbacks.read().unwrap();

    let Some(callbacks_for_type) = callbacks.get(&TypeId::of::<T>()) else {
        log::debug!("No callbacks registered");
        return;
    };

    log::debug!("Firing {} callbacks", callbacks_for_type.len());

    for cb in callbacks_for_type {
        cb(&event);
    }
}

/// Subscribes to a given event type with the given callback
#[profiling::function]
pub fn subscribe<T: WutEngineEvent>(callback: impl Fn(&T) + Send + Sync + 'static) {
    log::debug!("Subscribing to event of type {}", type_name::<T>());

    let mut callbacks = EVENT_MANAGER.callbacks.write().unwrap();

    let callbacks_for_type = callbacks.entry(TypeId::of::<T>()).or_default();

    callbacks_for_type.push(Box::new(move |untyped_event| {
        let untyped_as_any = untyped_event as &dyn Any;

        let typed = untyped_as_any
            .downcast_ref::<T>()
            .expect("Invalid event type given");

        callback(typed);
    }));
}

/// Initializes the global [EventManager]. Should be called once, and only once, by the WutEngine runtime.
/// Do not call manually
#[doc(hidden)]
pub fn init() {
    GlobalManager::init(&EVENT_MANAGER, EventManager::new());
}

pub(crate) struct EventManager {
    callbacks: RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn WutEngineEvent) + Send + Sync>>>>,
}

impl EventManager {
    fn new() -> Self {
        Self {
            callbacks: RwLock::new(HashMap::new()),
        }
    }
}

pub(crate) static EVENT_MANAGER: GlobalManager<EventManager> = GlobalManager::new();
