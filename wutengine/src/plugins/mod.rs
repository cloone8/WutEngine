use crate::command::Command;
use crate::runtime::RuntimeInitializer;

pub trait WutEnginePlugin: 'static {
    /// Called once right after [RuntimeInitializer::run] is called
    fn on_build(&mut self, _initializer: &mut RuntimeInitializer) {}

    /// Called once when the runtime has just been built, and is starting
    fn on_start(&mut self, _commands: &mut Command) {}
}
