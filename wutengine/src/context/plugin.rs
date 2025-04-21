use core::any::Any;

use crate::plugins::WutEnginePlugin;

/// The plugin context, used for interacting with loaded plugins
#[derive(Debug)]
pub struct PluginContext<'a> {
    plugins: &'a [Box<dyn WutEnginePlugin>],
}

impl<'a> PluginContext<'a> {
    /// Creates a new plugincontext containing the given plugins
    pub(crate) fn new(plugins: &'a [Box<dyn WutEnginePlugin>]) -> Self {
        Self { plugins }
    }

    /// Gets the instance of plugin type `T`, if loaded.
    pub fn get<T: WutEnginePlugin>(&self) -> Option<&T> {
        for plugin in self.plugins {
            let as_ref = plugin.as_ref() as &dyn Any;
            let cast = as_ref.downcast_ref::<T>();

            if let Some(cast_ok) = cast {
                return Some(cast_ok);
            }
        }

        None
    }
}
