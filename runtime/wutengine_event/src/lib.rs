#![doc = include_str!("../README.md")]

use core::any::Any;
use core::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use nohash_hasher::IntMap;
use wutengine_util::InitOnce;
use wutengine_util::assert_main_thread;
use wutengine_util_macro::unique_id_type32;

/// Type-erased event handler function
type GenericEventHandler = dyn Fn(&EventData) + Send + Sync;

/// The global [`EventManager`]
static EVENT_MANAGER: InitOnce<Mutex<EventManager>> = InitOnce::new_checked();

/// The event queue to send events to the [`EVENT_MANAGER`]
static EVENT_SENDER: InitOnce<Sender<EventData>> = InitOnce::new_checked();

/// Function that can be called to wake the main thread, which might cause it to
/// run the event loop
static WAKE_MAIN_THREAD_FN: InitOnce<Box<dyn Fn() + Send + Sync>> = InitOnce::new_checked();

/// The event manager. Holds a set of event subscribers and a cross-thread event queue
pub(crate) struct EventManager {
    /// The subscribers
    subscribers: IntMap<SubscriptionId, Box<GenericEventHandler>>,

    /// For each event type, holds a list of subscribers
    subscribers_by_type: HashMap<TypeId, Vec<SubscriptionId>>,

    /// The queue of new events
    event_queue_recv: Receiver<EventData>,
}

impl EventManager {
    /// A new [`EventManager`] and the queue to reach it
    fn new() -> (Self, Sender<EventData>) {
        let (send, recv) = std::sync::mpsc::channel();

        (
            Self {
                subscribers: HashMap::default(),
                subscribers_by_type: HashMap::default(),
                event_queue_recv: recv,
            },
            send,
        )
    }

    /// Adds a new subscriber
    fn add_subscriber(
        subscribers: &mut IntMap<SubscriptionId, Box<GenericEventHandler>>,
        subscribers_by_type: &mut HashMap<TypeId, Vec<SubscriptionId>>,
        data: AddSubscriber,
    ) {
        profiling::function_scope!();

        subscribers_by_type
            .entry(data.ty)
            .or_default()
            .push(data.id);
        subscribers.insert(data.id, data.handler);
    }

    /// Removes a subscriber
    fn remove_subscriber(
        subscribers: &mut IntMap<SubscriptionId, Box<GenericEventHandler>>,
        subscribers_by_type: &mut HashMap<TypeId, Vec<SubscriptionId>>,
        data: &RemoveSubscriber,
    ) {
        profiling::function_scope!();

        let handler = subscribers.remove(&data.0);
        assert!(handler.is_some(), "Removed unknown subscriber");

        subscribers_by_type
            .get_mut(&data.1)
            .expect("No TypeId entry for subscriber")
            .retain(|sub| sub != &data.0);
    }
}

/// Generic type-erased event data
#[derive(derive_more::Debug)]
struct EventData {
    /// [`TypeId`] of the event
    ty: TypeId,

    /// Type name of the event type
    ty_name: &'static str,

    /// Type-erased event
    #[debug(skip)]
    event: Box<dyn Event>,
}

impl EventData {
    /// Casts the inner event data to a concrete type and returns a reference. Must be valid
    fn get_as_type<T: Event>(&self) -> &T {
        assert_eq!(self.ty, TypeId::of::<T>(), "Invalid cast");

        let as_ref = self.event.as_ref() as &dyn Any;

        as_ref
            .downcast_ref::<T>()
            .expect("Invalid event data. Downcast should have succeeded")
    }

    /// Casts the inner event data to a concrete type. Must be valid
    fn take_as_type<T: Event>(self) -> T {
        assert_eq!(self.ty, TypeId::of::<T>(), "Invalid cast");

        *(self.event as Box<dyn Any>)
            .downcast()
            .expect("Invalid event data. Downcast should have succeeded")
    }
}

unique_id_type32! {
    /// Raw identifier of a [`Subscription`]
    SubscriptionId
}

/// Handle to a subscription to an event type. Dropping the handle does not cancel
/// the subscription, but makes it permanent
#[derive(Debug)]
pub struct Subscription(pub(crate) SubscriptionId, pub(crate) TypeId);

/// Add subscriber event. We should add it to the list of subscribers
struct AddSubscriber {
    /// The ID of the new subscriber
    id: SubscriptionId,

    /// The [`TypeId`] of the event
    ty: TypeId,

    /// The generic handler
    handler: Box<GenericEventHandler>,
}

/// Remove subscriber event. We should remove the subscription
struct RemoveSubscriber(SubscriptionId, TypeId);

/// A type that can be used as a WutEngine event
pub trait Event: Any + Send + Sync {}

impl<T: Any + Send + Sync> Event for T {}

/// Initializes the global event manager
#[doc(hidden)]
pub fn init(wake_main_thread_callback: impl Fn() + Send + Sync + 'static) {
    let (manager, sender) = EventManager::new();

    InitOnce::init(&EVENT_SENDER, sender);
    InitOnce::init(&EVENT_MANAGER, Mutex::new(manager));
    InitOnce::init(&WAKE_MAIN_THREAD_FN, Box::new(wake_main_thread_callback));
}

/// Publishes a new event. If any listeners are active, it will be
/// forwarded to their handler
#[inline]
pub fn publish<T: Event>(event: T) {
    let data = EventData {
        ty: TypeId::of::<T>(),
        ty_name: core::any::type_name::<T>(),
        event: Box::new(event),
    };

    log::debug!("Publishing new event of type {}", data.ty_name);

    EVENT_SENDER.send(data).expect("Event manager gone");
    WAKE_MAIN_THREAD_FN();
}

/// Subscribes to an event with the given handler. Returns a [`Subscription`] which can
/// be used to [`unsubscribe`] later.
#[inline]
pub fn subscribe<T: Event>(handler: impl Fn(&T) + Send + Sync + 'static) -> Subscription {
    let id = SubscriptionId::new();
    let ty = TypeId::of::<T>();

    log::debug!(
        "New subscriber {id} for event {}",
        core::any::type_name::<T>()
    );

    publish(AddSubscriber {
        id,
        ty,
        handler: Box::new(move |event_data| {
            let event_ref = event_data.get_as_type::<T>();

            handler(event_ref);
        }),
    });

    Subscription(id, ty)
}

/// Ubsubscribes from an event
#[inline]
#[expect(clippy::needless_pass_by_value, reason = "API should consume")]
pub fn unsubscribe(subscription: Subscription) {
    log::debug!("Subscriber {} unsubscribing", subscription.0);

    publish(RemoveSubscriber(subscription.0, subscription.1));
}

/// Handles any currently pending events. Automatically called by the engine runtime multiple times during a frame.
/// Must be called from the main thread.
pub fn handle_events() {
    profiling::function_scope!();
    assert_main_thread!();

    let mut event_manager_lock = EVENT_MANAGER.lock().unwrap(); // TODO: Main thread only. RefCell enough?
    let event_manager = &mut *event_manager_lock;

    for event in event_manager.event_queue_recv.try_iter() {
        profiling::scope!("Handle event", event.ty_name);

        log::trace!("Handling new event of type: {}", event.ty_name);

        if event.ty == TypeId::of::<AddSubscriber>() {
            EventManager::add_subscriber(
                &mut event_manager.subscribers,
                &mut event_manager.subscribers_by_type,
                event.take_as_type(),
            );
            continue;
        }

        if event.ty == TypeId::of::<RemoveSubscriber>() {
            EventManager::remove_subscriber(
                &mut event_manager.subscribers,
                &mut event_manager.subscribers_by_type,
                event.get_as_type(),
            );
            continue;
        }

        let Some(subscriber_ids) = event_manager.subscribers_by_type.get(&event.ty) else {
            continue;
        };

        for subscriber_id in subscriber_ids {
            let subscriber = event_manager
                .subscribers
                .get(subscriber_id)
                .expect("Missing subscriber. Invalid cleanup");

            subscriber(&event);
        }
    }
}
