use core::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;

use winit::event_loop::EventLoop;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::log::LogConfig;
use crate::plugins::WutEnginePlugin;
use crate::renderer::queue::RenderQueue;
use crate::renderer::shader_resolver::EmbeddedShaderResolver;
use crate::runtime::Runtime;
use crate::time::Time;
use crate::windowing::WindowingEvent;

use super::threadpool;

/// We only support starting and running a single runtime per
/// process. For that reason, we keep track of whether we've
/// already started a runtime once, and use that
/// to panic on trying to start a second one.
static RUNTIME_STARTED: AtomicBool = AtomicBool::new(false);

/// The main entry point for WutEngine.
///
/// Allows for engine configuration before actually starting the main
/// runtime (and yielding control to it.)
///
/// Configured using a builder pattern. See the various struct methods for
/// more specifics. To start, see [RuntimeInitializer::new].
#[derive(Default)]
pub struct RuntimeInitializer {
    log_config: LogConfig,
    plugins: Vec<Box<dyn WutEnginePlugin>>,
}

impl RuntimeInitializer {
    /// Creates a new, empty, runtime initializer
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the log configuration. Consecutive calls overwrite eachother.
    pub fn with_log_config(&mut self, config: LogConfig) -> &mut Self {
        self.log_config = config;
        self
    }

    /// Adds a new plugin to the engine. Consecutive calls add more plugins.
    pub fn with_plugin(&mut self, plugin: impl WutEnginePlugin) -> &mut Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    fn run_plugin_build_hooks(&mut self) {
        let mut all_plugins = Vec::new();

        while !self.plugins.is_empty() {
            // Drain the current plugin list, leaving it empty
            let mut plugins = std::mem::take(&mut self.plugins);

            for plugin in &mut plugins {
                plugin.on_build(self);
            }

            // Append the plugins to the accumulator, and loop
            // as the build hooks of the plugins might have added more plugins
            // themselves
            all_plugins.append(&mut plugins);
        }

        self.plugins.append(&mut all_plugins);
        self.plugins.shrink_to_fit();
    }

    /// Finalizes the runtime with the current configuration, and starts the
    /// WutEngine runtime with the given rendering backend.
    pub fn run<R: WutEngineRenderer>(mut self) {
        let runtime_already_started = RUNTIME_STARTED.swap(true, Ordering::SeqCst);

        if runtime_already_started {
            panic!("Another runtime has already been started, and WutEngine does not support multiple runtimes in the same process");
        }

        crate::log::initialize_loggers(&self.log_config);

        threadpool::init_threadpool();

        self.run_plugin_build_hooks();

        unsafe {
            Time::initialize();
        }

        let event_loop = EventLoop::<WindowingEvent>::with_user_event()
            .build()
            .unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut runtime = Runtime {
            identmap: HashMap::default(),
            objects: Vec::new(),
            render_queue: RenderQueue::new(),
            window_id_map: HashMap::new(),
            windows: HashMap::new(),
            eventloop: event_loop.create_proxy(),
            started: false,
            plugins: self.plugins,
            renderer: R::build(EmbeddedShaderResolver),
        };

        event_loop.run_app(&mut runtime).unwrap();
    }
}
