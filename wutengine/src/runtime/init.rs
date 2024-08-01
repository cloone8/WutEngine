use std::cell::UnsafeCell;
use std::collections::HashMap;

use nohash_hasher::IntMap;
use winit::event_loop::EventLoop;
use wutengine_core::{Component, ComponentTypeId};
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::log::LogConfig;
use crate::plugin::EnginePlugin;
use crate::renderer::shader_resolver::EmbeddedShaderResolver;
use crate::runtime::Runtime;
use crate::storage::{ComponentStorage, StorageKind};
use crate::{components, WindowingEvent};

#[derive(Default)]
pub struct RuntimeInitializer {
    plugins: Vec<Box<dyn EnginePlugin>>,
    components: IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
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

    pub fn add_component_type<T: Component>(&mut self) -> &mut Self {
        self.add_component_type_with_storage::<T>(StorageKind::Array)
    }

    pub fn add_component_type_with_storage<T: Component>(
        &mut self,
        storage: StorageKind,
    ) -> &mut Self {
        let id = T::COMPONENT_ID;

        if id <= ComponentTypeId::from_int(u16::MAX as u64) {
            panic!(
                "Trying to register component in builtin range! Given {}, min {}",
                id,
                (u16::MAX as u32) + 1
            );
        }

        if self.components.contains_key(&id) {
            panic!("Component already registered!");
        }

        self.components
            .insert(id, UnsafeCell::new(ComponentStorage::new_for::<T>(storage)));

        self
    }

    pub fn with_log_config(&mut self, config: LogConfig) -> &mut Self {
        self.log_config = config;
        self
    }

    pub(crate) fn add_builtin<T: Component>(&mut self, storage: StorageKind) -> &mut Self {
        let id = T::COMPONENT_ID;

        if self.components.contains_key(&id) {
            panic!("Component already registered!");
        }

        self.components
            .insert(id, UnsafeCell::new(ComponentStorage::new_for::<T>(storage)));

        self
    }

    pub fn run<R: WutEngineRenderer>(mut self) -> Result<(), ()> {
        crate::log::initialize_loggers(&self.log_config);

        components::register_builtins(&mut self);

        let event_loop = EventLoop::<WindowingEvent>::with_user_event()
            .build()
            .unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        self.components.shrink_to_fit();

        let mut runtime = Runtime {
            plugins: self.plugins.into_boxed_slice(),
            entities: Vec::new(),
            components: self.components,
            systems: Vec::new(),
            window_id_map: HashMap::new(),
            windows: HashMap::new(),
            eventloop: event_loop.create_proxy(),
            started: false,
            renderer: R::build(EmbeddedShaderResolver),
        };

        event_loop.run_app(&mut runtime).unwrap();

        Ok(())
    }
}
