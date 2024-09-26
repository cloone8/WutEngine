use std::collections::HashMap;

use winit::event_loop::EventLoop;
use wutengine_ecs::world::World;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::log::LogConfig;
use crate::plugin::EnginePlugin;
use crate::renderer::shader_resolver::EmbeddedShaderResolver;
use crate::runtime::Runtime;
use crate::WindowingEvent;

#[derive(Default)]
pub struct RuntimeInitializer {
    plugins: Vec<Box<dyn EnginePlugin>>,
    log_config: LogConfig,
}

impl RuntimeInitializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plugin<P: EnginePlugin>(&mut self) -> &mut Self {
        self.plugins.push(Box::new(P::build()));
        self
    }

    pub fn with_log_config(&mut self, config: LogConfig) -> &mut Self {
        self.log_config = config;
        self
    }

    pub fn run<R: WutEngineRenderer>(self) {
        crate::log::initialize_loggers(&self.log_config);

        let event_loop = EventLoop::<WindowingEvent>::with_user_event()
            .build()
            .unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut runtime = Runtime {
            plugins: self.plugins.into_boxed_slice(),
            world: World::default(),
            systems: Vec::new(),
            window_id_map: HashMap::new(),
            windows: HashMap::new(),
            eventloop: event_loop.create_proxy(),
            started: false,
            renderer: R::build(EmbeddedShaderResolver),
        };

        event_loop.run_app(&mut runtime).unwrap();
    }
}
