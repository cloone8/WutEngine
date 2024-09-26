use std::collections::HashMap;

use winit::event_loop::EventLoop;
use wutengine_core::{System, SystemPhase};
use wutengine_ecs::world::World;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::command::Command;
use crate::ecs::FunctionDescription;
use crate::log::LogConfig;
use crate::plugins::WutEnginePlugin;
use crate::renderer::shader_resolver::EmbeddedShaderResolver;
use crate::runtime::Runtime;
use crate::WindowingEvent;

#[derive(Default)]
pub struct RuntimeInitializer {
    log_config: LogConfig,
    plugins: Vec<Box<dyn WutEnginePlugin>>,
    systems: Vec<System<World, Command>>,
}

impl RuntimeInitializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_log_config(&mut self, config: LogConfig) -> &mut Self {
        self.log_config = config;
        self
    }

    pub fn with_plugin(&mut self, plugin: impl WutEnginePlugin) -> &mut Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn with_system<T: FunctionDescription>(&mut self, phase: SystemPhase) -> &mut Self {
        let descriptor = T::describe();

        self.systems.push(System {
            phase,
            read_writes: descriptor.read_writes,
            func: descriptor.func,
        });

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
    }

    pub fn run<R: WutEngineRenderer>(mut self) {
        self.run_plugin_build_hooks();

        crate::log::initialize_loggers(&self.log_config);

        let event_loop = EventLoop::<WindowingEvent>::with_user_event()
            .build()
            .unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut runtime = Runtime {
            world: World::default(),
            systems: self.systems,
            window_id_map: HashMap::new(),
            windows: HashMap::new(),
            eventloop: event_loop.create_proxy(),
            started: false,
            renderer: R::build(EmbeddedShaderResolver),
        };

        event_loop.run_app(&mut runtime).unwrap();
    }
}
