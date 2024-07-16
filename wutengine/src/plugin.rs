use crate::{EngineCommand, EngineEvent};

pub trait EnginePlugin {
    #[must_use]
    fn on_event(&mut self, event: &EngineEvent) -> Vec<EngineCommand>;
}
