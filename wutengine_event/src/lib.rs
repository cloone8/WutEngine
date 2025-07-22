//! WutEngine event handling and dispatching

use core::any::Any;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender};

use wutengine_util::{GlobalManager, TypeName};

#[derive(Debug)]
pub struct EventManager {
    event_recv: Mutex<Receiver<Box<dyn WutEngineEvent>>>,
    event_send: Sender<Box<dyn WutEngineEvent>>,
}

impl EventManager {
    fn new() -> Self {
        let (send, recv) = std::sync::mpsc::channel::<Box<dyn WutEngineEvent>>();
        Self {
            event_recv: Mutex::new(recv),
            event_send: send,
        }
    }
}

static EVENT_MANAGER: GlobalManager<EventManager> = GlobalManager::new();

pub fn init() {
    GlobalManager::init(&EVENT_MANAGER, EventManager::new());
}

pub trait WutEngineEvent: Any + TypeName + Send + Sync {}

pub fn publish<T: WutEngineEvent>(event: T) {
    log::debug!("Publishing event of type {}", event.type_name());

    EVENT_MANAGER
        .event_send
        .send(Box::new(event))
        .expect("Failed to publish WutEngine event");
}

pub fn handle_pending_events() -> bool {
    let event_recv = EVENT_MANAGER.event_recv.lock().unwrap();

    let mut any_event = false;

    for event in event_recv.try_iter() {
        log::info!("Fake-handling event of type {}", event.as_ref().type_name());

        any_event = true;
    }

    any_event
}
