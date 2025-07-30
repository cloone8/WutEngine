//! Event subscriptions. Re-exported through the main lib file

use core::any::TypeId;
use std::sync::Arc;

use crate::EVENT_MANAGER;

wutengine_util::unique_id_type64!(
    /// The ID of a subscription to an event type
    pub(crate) EventSubscriptionId
);

/// A subscription to a [WutEngineEvent](crate::WutEngineEvent). Internally ref counted, so cheap to clone. Once the last reference
/// to the subscription is dropped, automatically unsubscribes from the event
#[derive(Debug, Clone)]
#[must_use = "Subscriptions only last while their IDs exist"]
#[repr(transparent)]
pub struct EventSubscription(pub(crate) Arc<EventSubscriptionInner>);

/// The inner [EventSubscription]. Unsubscribes when dropped
#[derive(Debug)]
pub(crate) struct EventSubscriptionInner {
    /// The type of the event
    pub(crate) event_type: TypeId,

    /// The subscription ID
    pub(crate) subscriber_id: EventSubscriptionId,
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
