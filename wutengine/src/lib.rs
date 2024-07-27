use std::rc::Rc;
use std::{cell::UnsafeCell, collections::HashMap};

use command::{Command, OpenWindowParams};
use components::{camera::Camera, mesh::Mesh, ID_CAMERA};
use nohash_hasher::IntMap;
use plugin::EnginePlugin;
use storage::{ComponentStorage, StorageKind};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

#[doc(inline)]
pub use wutengine_core as core;

#[doc(inline)]
pub use wutengine_graphics as graphics;
use wutengine_graphics::material::MaterialData;
use wutengine_graphics::shader::builtins::UNLIT;
use wutengine_graphics::shader::ShaderVariant;

use wutengine_graphics::{
    renderer::{Renderable, WutEngineRenderer},
    windowing::WindowIdentifier,
};

#[doc(inline)]
pub use wutengine_macro as macros;

use world::{Queryable, World};

use wutengine_core::{
    EntityId, {Component, ComponentTypeId, DynComponent}, {System, SystemPhase},
};

pub mod command;
pub mod components;
pub mod math;
pub mod plugin;
pub mod renderer;
pub mod storage;
pub mod world;

#[derive(Debug)]
pub enum SystemFunction {
    Immutable(for<'a> fn(&mut Command, &'a World<'a>)),
    Mutable(for<'a> fn(&mut Command, &'a mut World<'a>)),
}

#[derive(Debug)]
pub enum EngineCommand {
    AddSystem(System<SystemFunction>),
    SpawnEntity(EntityId, Vec<Box<dyn DynComponent>>),
    OpenWindow(OpenWindowParams),
}

#[derive(Debug)]
pub enum WindowingEvent {
    OpenWindow(OpenWindowParams),
}

#[derive(Debug)]
pub enum EngineEvent {
    RuntimeStart,
}

#[derive(Default)]
pub struct RuntimeInitializer {
    plugins: Vec<Box<dyn EnginePlugin>>,
    components: IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
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
            renderer: R::default(),
        };

        event_loop.run_app(&mut runtime).unwrap();

        Ok(())
    }
}

pub struct Runtime<R: WutEngineRenderer> {
    plugins: Box<[Box<dyn EnginePlugin>]>,

    entities: Vec<EntityId>,
    components: IntMap<ComponentTypeId, UnsafeCell<ComponentStorage>>,
    systems: Vec<System<SystemFunction>>,

    eventloop: EventLoopProxy<WindowingEvent>,

    window_id_map: HashMap<WindowId, WindowIdentifier>,
    windows: HashMap<WindowIdentifier, Window>,

    started: bool,

    renderer: R,
}

impl<R: WutEngineRenderer> Runtime<R> {
    unsafe fn get_component_for_entity<T: Component>(&self, entity: EntityId) -> Option<&T> {
        if let Some(storage) = self.components.get(&T::COMPONENT_ID) {
            let storage_cell = storage.get();
            let storage = storage_cell.as_ref().expect("Storage returned nullptr");

            return storage.get::<T>(entity);
        }

        None
    }

    /// # Safety
    ///
    /// The components you are querying for _must_ not be accessed mutable by more
    /// than one caller at a time.
    unsafe fn query<'a, T: Queryable<'a>>(&'a self) -> Vec<(EntityId, Option<T>)> {
        T::do_query(&self.entities, &self.components)
    }

    fn get_renderables(&self) -> Vec<Renderable> {
        let query_result: Vec<(EntityId, Option<&Mesh>)> = unsafe { self.query() };

        let mut renderables = Vec::new();

        for components in query_result
            .into_iter()
            .filter(|(_, comps)| comps.is_some())
            .map(|(_, comps)| comps.unwrap())
        {
            log::trace!("Pushing renderable mesh: {:#?}", components);

            renderables.push(Renderable {
                mesh: components.data.clone(),
                material: Rc::new(MaterialData {
                    shader: UNLIT.make_variant(Vec::<String>::new()),
                }),
            })
        }

        renderables
    }

    fn exec_engine_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::AddSystem(system) => self.systems.push(system),
            EngineCommand::OpenWindow(params) => self
                .eventloop
                .send_event(WindowingEvent::OpenWindow(params))
                .unwrap(),
            EngineCommand::SpawnEntity(id, components) => {
                debug_assert!(!self.entities.contains(&id));
                self.entities.push(id);

                for component in components.into_iter() {
                    let array = self
                        .components
                        .get_mut(&component.get_dyn_component_id())
                        .expect("Unknown component type!")
                        .get_mut();

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
        log::debug!("Sending engine event:\n{:#?}", event);

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
            let mut world = unsafe { World::new(&self.entities, &self.components) };

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

impl<R: WutEngineRenderer> ApplicationHandler<WindowingEvent> for Runtime<R> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.started {
            log::info!("Initializing WutEngine");
            self.start();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if !self.started {
            log::trace!("about_to_wait fired but engine not yet initialized");
            return;
        }

        log::trace!("Starting frame");

        self.run_systems_for_phase(SystemPhase::Update);

        let cam_storage = unsafe {
            self.components
                .get(&ID_CAMERA)
                .unwrap()
                .get()
                .as_ref()
                .unwrap()
        };

        let all_cams = cam_storage.all::<Camera>();

        let renderables = self.get_renderables();

        for camera in all_cams {
            if !self.windows.contains_key(&camera.component.display) {
                log::warn!(
                    "Camera trying to render to non-existing window {}",
                    &camera.component.display
                );
                continue;
            }

            self.renderer
                .render(camera.component.to_context(), &renderables);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WindowingEvent) {
        log::debug!("Handling WutEngine WindowingEvent:\n{:#?}", event);

        match event {
            WindowingEvent::OpenWindow(params) => {
                if self.windows.contains_key(&params.id) {
                    if params.ignore_existing {
                    } else {
                        panic!("Window {} already exists!", params.id);
                    }
                }

                let attrs = Window::default_attributes()
                    .with_title(params.title)
                    .with_fullscreen(params.mode.into());

                let window = event_loop.create_window(attrs).unwrap();

                self.renderer
                    .new_window(&params.id, &window, window.inner_size().into());

                let old_val = self.window_id_map.insert(window.id(), params.id.clone());

                debug_assert!(old_val.is_none());

                let old_val = self.windows.insert(params.id, window);

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
        let identifier = self.window_id_map.get(&window_id).unwrap().clone();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                log::debug!(
                    "Resizing window {} to {}x{}",
                    identifier,
                    size.width,
                    size.height
                );

                self.renderer.size_changed(&identifier, size.into());

                if cfg!(target_os = "windows") {
                    // hack for resizing bug in winit, remove once fixed
                    self.about_to_wait(event_loop);
                }
            }
            _ => (),
        }
    }
}
