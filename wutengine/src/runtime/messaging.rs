//! Cross-object message passing

use core::any::Any;
use core::fmt::Debug;
use std::sync::{Arc, Mutex};

use crate::gameobject::GameObjectId;

/// A message queue, holding the messages that have been sent by components, but that
/// have not yet been handled
#[derive(Debug)]
pub(crate) struct MessageQueue {
    messages: Mutex<Vec<MessageData>>,
}

impl MessageQueue {
    /// Returns whether the queue currently contains any elements.
    /// Requires exclusive access to prevent mistakes assuming the queue
    /// is empty while another tread is still trying to fill it.
    pub(crate) fn is_empty(&mut self) -> bool {
        let locked = self.messages.get_mut().unwrap();

        locked.is_empty()
    }
}

#[derive(Debug)]
struct MessageData {
    message: Message,
    target: MessageTarget,
}

/// A message sent from a GameObject
#[derive(Debug, Clone)]
pub struct Message {
    content: Arc<dyn MessageCompatible>,
}

/// A marker trait signalling that a type is safe to send as a cross-gameobject message.
/// Implemented automatically if the required traits [Any], [Send], [Sync] and [Debug]
/// are also implemented.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not message-compatible. It needs to implement all of `Any`, `Send`, `Sync` and `Debug`"
)]
pub trait MessageCompatible: Any + Send + Sync + Debug {
    /// Casts the message to an [Any] reference, for internal conversions.
    fn as_any(&self) -> &dyn Any;
}

impl<T> MessageCompatible for T
where
    T: Any + Send + Sync + Debug,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Message {
    /// Creates a new message with the given content
    pub(crate) fn new<T: MessageCompatible>(content: T) -> Self {
        Self {
            content: Arc::new(content),
        }
    }

    /// Tries to get the message content as a certain type
    pub fn try_cast<T: MessageCompatible>(&self) -> Option<&T> {
        // Make sure to extract the reference _out_ of the Arc first,
        // to prevent accidentally casting the Arc itself instead
        // of the content
        let arc_inner = self.content.as_ref();

        arc_inner.as_any().downcast_ref::<T>()
    }
}

/// The target for a given message
#[derive(Debug, Clone, Copy)]
pub enum MessageTarget {
    /// Global message sent to all currently loaded GameObjects
    Global,

    /// Message sent to a specific GameObject
    GameObject(GameObjectId),
}

impl MessageQueue {
    /// Creates a new, empty, [MessageQueue]
    pub(crate) fn new() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
        }
    }

    /// Adds a new message to the queue
    pub(crate) fn add_message(&self, message: Message, target: MessageTarget) {
        let data = MessageData { message, target };

        let mut locked = self.messages.lock().unwrap();

        locked.push(data);
    }

    /// Returns the references to the messages targeted at the given GameObject
    pub(crate) fn get_messages_for(&self, target: GameObjectId, outbuf: &mut Vec<Message>) {
        debug_assert!(outbuf.is_empty(), "Output buffer not empty yet!");

        let locked = self.messages.lock().unwrap();

        for msg in locked.iter() {
            let target_matches = match msg.target {
                MessageTarget::Global => true,
                MessageTarget::GameObject(msg_target) => msg_target == target,
            };

            if target_matches {
                outbuf.push(msg.message.clone())
            }
        }
    }
}
