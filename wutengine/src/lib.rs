use componentset::ComponentSet;
use nohash_hasher::IntMap;
use plugin::EnginePlugin;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::WindowId,
};
use wutengine_core::{ComponentTypeId, System, SystemFunction, SystemPhase, World};

pub use wutengine_core as core;
pub use wutengine_macro as macros;

mod componentset;
pub mod plugin;

pub enum EngineCommand {
    AddSystem(System),
}

pub enum WindowingEvent {}

pub enum EngineEvent {
    RuntimeStart,
}

pub struct RuntimeInitializer {
    plugins: Vec<Box<dyn EnginePlugin>>,
}

impl RuntimeInitializer {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn add_plugin(&mut self, plugin: Box<dyn EnginePlugin>) -> &mut Self {
        self.plugins.push(plugin);
        self
    }

    pub fn run(mut self) -> Result<(), ()> {
        let event_loop = EventLoop::<WindowingEvent>::with_user_event()
            .build()
            .unwrap();

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let mut runtime = Runtime {
            plugins: self.plugins.into_boxed_slice(),
            components: IntMap::default(),
            systems: Vec::new(),
            eventloop: event_loop.create_proxy(),
            started: false,
        };

        event_loop.run_app(&mut runtime).unwrap();

        Ok(())
    }
}

pub struct Runtime {
    plugins: Box<[Box<dyn EnginePlugin>]>,
    components: IntMap<ComponentTypeId, ComponentSet>,
    systems: Vec<System>,

    eventloop: EventLoopProxy<WindowingEvent>,

    started: bool,
}

impl Runtime {
    fn exec_engine_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::AddSystem(system) => self.systems.push(system),
        }
    }

    fn send_engine_event(&mut self, event: EngineEvent) {
        let mut response_commands: Vec<EngineCommand> = Vec::new();

        for plugin in self.plugins.iter_mut() {
            let response = plugin.on_event(&event);
            response_commands.extend(response.into_iter());
        }

        for command in response_commands {
            self.exec_engine_command(command);
        }
    }

    fn run_systems_for_phase(&mut self, phase: SystemPhase) {
        for system in self.systems.iter_mut().filter(|sys| sys.phase == phase) {
            let mut world = World;

            match system.func {
                SystemFunction::Immutable(func) => func(&world),
                SystemFunction::Mutable(func) => func(&mut world),
            }
        }
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

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WindowingEvent) {}

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
