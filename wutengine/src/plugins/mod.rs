use crate::runtime::RuntimeInitializer;

pub trait WutEnginePlugin: 'static {
    /// Called once right after [RuntimeInitializer::run] is called
    fn on_build(&mut self, initializer: &mut RuntimeInitializer);
}
