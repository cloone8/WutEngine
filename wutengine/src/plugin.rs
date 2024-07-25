use crate::{EngineCommand, EngineEvent};

pub trait EnginePlugin: 'static {
    fn build() -> Self
    where
        Self: Sized;

    #[must_use]
    fn on_event(&mut self, event: &EngineEvent) -> Vec<EngineCommand>;
}
