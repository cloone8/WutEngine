use std::any::Any;

pub trait Event: Clone + Any + Sized {
    const ALLOW_MULTIPLE: bool;
    const LIFETIME: EventLifetime;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLifetime {
    UntilNew,
    NextFrame,
}
