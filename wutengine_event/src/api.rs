//! Public event API. Re-exported through the main lib file

use core::any::Any;
use std::sync::Arc;

use wutengine_util::{GlobalManager, TypeName};

use crate::{
    EVENT_MANAGER, EventManager, EventSubscription, EventSubscriptionId, EventSubscriptionInner,
};

/// Initializes the global event manager. Must be called before any event is published or subscribed to
pub fn init() {
    GlobalManager::init(&EVENT_MANAGER, EventManager::new());
}

/// Trait implemented by types that want to be published or subscribed to as events
pub trait WutEngineEvent: Any + TypeName + Send + Sync {}

/// Publishes a new event into the global event queue. Event handling order is not completely deterministic.
#[profiling::function]
pub fn publish<T: WutEngineEvent>(event: T) {
    log::debug!("Publishing event of type {}", event.type_name());

    EVENT_MANAGER
        .event_send
        .send(Box::new(event))
        .expect("Failed to publish WutEngine event");
}

/// Handles any currently pending event. Should only be called by event handling implementations, not by users.
/// Returns `true` if any events were handled
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

/// Subscribes to a given [WutEngineEvent] type. Once an event of this type is published and handled,
/// `callback` is called with a reference to the specific event instance.
///
/// The subscription to the event lasts only as long as the returned [EventSubscription]. For permanent subscriptions, see [subscribe_permanent]
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

/// Same as [subscribe], but the subscription is permanent. Useful only for global managers and other callbacks that need to live
/// as long as the process itself
#[profiling::function]
pub fn subscribe_permanent<T: WutEngineEvent>(callback: impl Fn(&T) + Send + Sync + 'static) {
    let subscription = subscribe(callback);

    // Forget the subscription without running its destructor, which unsubscribes.
    // Take it out of the Arc first so we do not leak heap memory
    let inner = Arc::into_inner(subscription.0)
        .expect("Fresh subscription should have only one subscriber");

    core::mem::forget(inner);
}
