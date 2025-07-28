//! WutEngine event handling and dispatching
//!
//! Provides a global crate where any other WutEngine sub-crate (and by extension, WutEngine users) can hook into and publish new events.

use core::any::TypeId;
use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use wutengine_util::GlobalManager;
use wutengine_util::hash::nohash_hasher::{IntMap, IntSet};

mod api;
mod subscription;

pub use api::*;
pub use subscription::*;

/// The global [EventManager]
static EVENT_MANAGER: GlobalManager<EventManager> = GlobalManager::new();

/// The main event manager. Publishes events to a global channel, and then receives them
/// and distributes them to subscribers once polled with [handle_pending_events].
#[derive(Debug)]
struct EventManager {
    /// Receiver channel. Only read through [handle_pending_events]
    event_recv: Mutex<Receiver<Box<dyn WutEngineEvent>>>,

    /// Event sender channel
    event_send: Sender<Box<dyn WutEngineEvent>>,

    /// Subscriber info
    subscribers: Mutex<Subscribers>,
}

/// A dynamic (untyped) event callback. Type checking and casting is probably done within the callback
type DynamicEventCallback = dyn Fn(&dyn WutEngineEvent) + Send + Sync;

/// All event subscribers known to a single [EventManager]
struct Subscribers {
    /// A mapping between event [TypeId] and subscribers, to make handling events more performant
    event_type_subscribers: HashMap<TypeId, IntSet<EventSubscriptionId>>,

    /// Contains the callbacks for each actual subscription
    subscribers: IntMap<EventSubscriptionId, Arc<DynamicEventCallback>>,
}

impl Subscribers {
    /// A new, empty, [Subscribers] map
    fn new() -> Self {
        Self {
            event_type_subscribers: HashMap::default(),
            subscribers: HashMap::default(),
        }
    }
}

impl Debug for Subscribers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Subscribers").finish()
    }
}

impl EventManager {
    /// A new, empty, [EventManager]
    fn new() -> Self {
        let (send, recv) = std::sync::mpsc::channel::<Box<dyn WutEngineEvent>>();
        Self {
            event_recv: Mutex::new(recv),
            event_send: send,
            subscribers: Mutex::new(Subscribers::new()),
        }
    }
}
