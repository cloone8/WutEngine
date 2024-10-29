use crate::gameobject::GameObjectId;
use crate::runtime::messaging::{Message, MessageCompatible, MessageQueue, MessageTarget};

/// The message context. Used for interacting with the WutEngine message-passing API.
#[derive(Debug)]
pub struct MessageContext<'a> {
    queue: &'a MessageQueue,
}

impl<'a> MessageContext<'a> {
    /// Creates a new MessageContext with the given queue
    pub(crate) fn new(queue: &'a MessageQueue) -> Self {
        Self { queue }
    }

    /// Sends the given message to the given target
    pub fn send<T: MessageCompatible>(&self, message: T, target: MessageTarget) {
        self.queue.add_message(Message::new(message), target);
    }

    /// Sends the given message globally
    pub fn send_global<T: MessageCompatible>(&self, message: T) {
        self.send(message, MessageTarget::Global);
    }

    /// Sends the given message to the given GameObject
    pub fn send_gameobject<T: MessageCompatible>(&self, message: T, gameobject: GameObjectId) {
        self.send(message, MessageTarget::GameObject(gameobject));
    }
}
