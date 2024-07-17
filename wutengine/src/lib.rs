use std::collections::HashMap;

use command::Command;
use component::storage::array::ComponentArray;
use nohash_hasher::IntMap;
use plugin::EnginePlugin;
use windowing::WindowIdentifier;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use world::World;
pub use wutengine_core as core;
use wutengine_core::{
    component::{Component, ComponentTypeId, DynComponent},
    entity::EntityId,
    system::{System, SystemPhase},
};
pub use wutengine_macro as macros;

pub mod command;
pub mod component;
pub mod plugin;
pub mod windowing;
pub mod world;

#[derive(Debug)]
pub enum SystemFunction {
    Immutable(fn(&mut Command, &World)),
    Mutable(fn(&mut Command, &mut World)),
}

#[derive(Debug)]
pub enum EngineCommand {
    AddSystem(System<SystemFunction>),
    SpawnEntity(EntityId, Vec<Box<dyn DynComponent>>),
    OpenWindow(WindowIdentifier),
}

#[derive(Debug)]
pub enum WindowingEvent {
    OpenWindow(WindowIdentifier),
}

pub enum EngineEvent {
    RuntimeStart,
}

pub struct RuntimeInitializer {
    plugins: Vec<Box<dyn EnginePlugin>>,
    components: IntMap<ComponentTypeId, ComponentArray>,
}

impl RuntimeInitializer {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            components: IntMap::default(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Box<dyn EnginePlugin>) -> &mut Self {
        self.plugins.push(plugin);
        self
    }

    pub fn add_component_type<T: Component>(&mut self) -> &mut Self {
        let id = T::get_component_id();

        if self.components.contains_key(&id) {
            panic!("Component already registered!");
        }

        self.components.insert(id, ComponentArray::new_for::<T>());

        self
    }

    pub fn run(mut self) -> Result<(), ()> {
        let event_loop = EventLoop::<WindowingEvent>::with_user_event()
            .build()
            .unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        self.components.shrink_to_fit();

        let mut runtime = Runtime {
            plugins: self.plugins.into_boxed_slice(),
            components: self.components,
            systems: Vec::new(),
            windows: HashMap::new(),
            eventloop: event_loop.create_proxy(),
            started: false,
        };

        event_loop.run_app(&mut runtime).unwrap();

        Ok(())
    }
}

pub struct Runtime {
    plugins: Box<[Box<dyn EnginePlugin>]>,
    components: IntMap<ComponentTypeId, ComponentArray>,
    systems: Vec<System<SystemFunction>>,

    eventloop: EventLoopProxy<WindowingEvent>,
    windows: HashMap<WindowIdentifier, Window>,

    started: bool,
}

impl Runtime {
    fn exec_engine_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::AddSystem(system) => self.systems.push(system),
            EngineCommand::OpenWindow(id) => self
                .eventloop
                .send_event(WindowingEvent::OpenWindow(id))
                .unwrap(),
            EngineCommand::SpawnEntity(id, components) => {
                for component in components.into_iter() {
                    let array = self
                        .components
                        .get_mut(&component.get_dyn_component_id())
                        .expect("Unknown component type!");

                    array.push(id, component);
                }
            }
        }
    }

    fn exec_engine_commands(&mut self, commands: impl IntoIterator<Item = EngineCommand>) {
        for command in commands {
            self.exec_engine_command(command);
        }
    }

    fn send_engine_event(&mut self, event: EngineEvent) {
        let mut response_commands: Vec<EngineCommand> = Vec::new();

        for plugin in self.plugins.iter_mut() {
            let response = plugin.on_event(&event);
            response_commands.extend(response.into_iter());
        }

        self.exec_engine_commands(response_commands);
    }

    fn run_systems_for_phase(&mut self, phase: SystemPhase) {
        let mut commands = Command::empty();

        for system in self.systems.iter_mut().filter(|sys| sys.phase == phase) {
            let mut world = World;

            match system.func {
                SystemFunction::Immutable(func) => func(&mut commands, &world),
                SystemFunction::Mutable(func) => func(&mut commands, &mut world),
            }
        }

        self.exec_engine_commands(commands.consume());
    }

    fn start(&mut self) {
        self.send_engine_event(EngineEvent::RuntimeStart);
        self.run_systems_for_phase(SystemPhase::RuntimeStart);

        self.started = true;
    }
}

impl ApplicationHandler<WindowingEvent> for Runtime {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.started {
            self.start();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.run_systems_for_phase(SystemPhase::Update);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WindowingEvent) {
        match event {
            WindowingEvent::OpenWindow(id) => {
                let window = event_loop
                    .create_window(Window::default_attributes())
                    .unwrap();

                let old_val = self.windows.insert(id, window);

                debug_assert!(old_val.is_none());
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => (),
        }
    }
}
