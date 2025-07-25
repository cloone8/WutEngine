//! WutEngine event handling and dispatching

use core::any::{Any, TypeId};
use core::f32::consts::E;
use core::fmt::Debug;
use core::sync::atomic::AtomicU64;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use wutengine_util::hash::nohash_hasher::{IntMap, IntSet};
use wutengine_util::{GlobalManager, TypeName};

wutengine_util::unique_id_type!(EventSubscriptionId);

#[derive(Debug)]
pub struct EventManager {
    event_recv: Mutex<Receiver<Box<dyn WutEngineEvent>>>,
    event_send: Sender<Box<dyn WutEngineEvent>>,
    subscribers: Mutex<Subscribers>,
}

struct Subscribers {
    event_type_subscribers: HashMap<TypeId, IntSet<EventSubscriptionId>>,
    subscribers: IntMap<EventSubscriptionId, Arc<dyn Fn(&dyn WutEngineEvent) + Send + Sync>>,
}

impl Subscribers {
    fn new() -> Self {
        Self {
            event_type_subscribers: HashMap::default(),
            subscribers: HashMap::default(),
        }
    }
}

impl Debug for Subscribers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscribers").finish()
    }
}

impl EventManager {
    fn new() -> Self {
        let (send, recv) = std::sync::mpsc::channel::<Box<dyn WutEngineEvent>>();
        Self {
            event_recv: Mutex::new(recv),
            event_send: send,
            subscribers: Mutex::new(Subscribers::new()),
        }
    }
}

static EVENT_MANAGER: GlobalManager<EventManager> = GlobalManager::new();

pub fn init() {
    GlobalManager::init(&EVENT_MANAGER, EventManager::new());
}

pub trait WutEngineEvent: Any + TypeName + Send + Sync {}

#[profiling::function]
pub fn publish<T: WutEngineEvent>(event: T) {
    log::debug!("Publishing event of type {}", event.type_name());

    EVENT_MANAGER
        .event_send
        .send(Box::new(event))
        .expect("Failed to publish WutEngine event");
}

#[profiling::function]
pub fn handle_pending_events() -> bool {
    let event_recv = EVENT_MANAGER.event_recv.lock().unwrap();

    let mut any_event = false;

    let mut to_call = Vec::with_capacity(8);

    for event in event_recv.try_iter() {
        log::info!("Handling event of type {}", event.as_ref().type_name());

        let event_type_id = (event.as_ref() as &dyn Any).type_id();

        let subscribers = EVENT_MANAGER.subscribers.lock().unwrap();

        let Some(type_subscriber_ids) = subscribers.event_type_subscribers.get(&event_type_id)
        else {
            continue;
        };

        for subscriber_id in type_subscriber_ids {
            let subscriber_callback = subscribers
                .subscribers
                .get(subscriber_id)
                .expect("Missing event subscriber");

            to_call.push(Arc::clone(subscriber_callback));
        }

        drop(subscribers); // Drop subscribers so that new subscriptions do not deadlock

        for callback in to_call.drain(..) {
            callback(event.as_ref());
        }

        any_event = true;
    }

    any_event
}

#[profiling::function]
pub fn subscribe<T: WutEngineEvent>(
    callback: impl Fn(&T) + Send + Sync + 'static,
) -> EventSubscription {
    let subscription_id = EventSubscriptionId::new();

    let mut subscribers = EVENT_MANAGER.subscribers.lock().unwrap();

    subscribers
        .event_type_subscribers
        .entry(core::any::TypeId::of::<T>())
        .or_default()
        .insert(subscription_id);

    subscribers.subscribers.insert(
        subscription_id,
        Arc::new(move |untyped_event| {
            log::debug!(
                "Notifying subscriber {subscription_id} of event of type {}",
                core::any::type_name::<T>()
            );

            let typed_event = (untyped_event as &dyn Any)
                .downcast_ref::<T>()
                .expect("Invalid event type cast. Engine error");

            callback(typed_event);
        }),
    );

    EventSubscription(Arc::new(EventSubscriptionInner {
        event_type: core::any::TypeId::of::<T>(),
        subscriber_id: subscription_id,
    }))
}

#[derive(Debug, Clone)]
#[must_use = "Subscriptions only last while their IDs exist"]
#[repr(transparent)]
pub struct EventSubscription(Arc<EventSubscriptionInner>);

#[derive(Debug)]
struct EventSubscriptionInner {
    event_type: TypeId,
    subscriber_id: EventSubscriptionId,
}

impl Drop for EventSubscriptionInner {
    #[profiling::function]
    fn drop(&mut self) {
        log::debug!("Unsubscribing subscription {}", self.subscriber_id);

        let mut subscribers = EVENT_MANAGER.subscribers.lock().unwrap();

        let Some(event_type_subscribers) =
            subscribers.event_type_subscribers.get_mut(&self.event_type)
        else {
            return;
        };

        event_type_subscribers.remove(&self.subscriber_id);
        subscribers.subscribers.remove(&self.subscriber_id);
    }
}
