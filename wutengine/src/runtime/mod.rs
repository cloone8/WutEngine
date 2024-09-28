use std::collections::HashMap;

use glam::Mat4;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use winit::{application::ApplicationHandler, dpi::PhysicalSize};
use wutengine_core::{System, SystemPhase};
use wutengine_ecs::world::World;
use wutengine_graphics::renderer::{Renderable, WutEngineRenderer};
use wutengine_graphics::windowing::WindowIdentifier;

use crate::builtins::components::Material;
use crate::builtins::components::Mesh;
use crate::builtins::components::{Camera, Transform};
use crate::command::Command;
use crate::plugins::WutEnginePlugin;
use crate::{EngineCommand, WindowingEvent};

mod init;

pub use init::*;

/// The main runtime for WutEngine. Cannot be constructed directly. Instead,
/// construct a runtime with a [RuntimeInitializer]
pub struct Runtime<R: WutEngineRenderer> {
    world: World,
    systems: Vec<System<World, Command>>,

    eventloop: EventLoopProxy<WindowingEvent>,

    window_id_map: HashMap<WindowId, WindowIdentifier>,
    windows: HashMap<WindowIdentifier, Window>,

    started: bool,

    plugins: Vec<Box<dyn WutEnginePlugin>>,
    renderer: R,
}

impl<R: WutEngineRenderer> Runtime<R> {
    unsafe fn get_renderables(&self) -> Vec<Renderable> {
        unsafe {
            self.world
                .query(|id, args: (&Mesh, &Material, Option<&Transform>)| {
                    let mesh = args.0.data.clone();
                    let material = args.1.data.clone();

                    let transform = if let Some(transform) = args.2 {
                        transform.local_to_world()
                    } else {
                        log::trace!(
                            "Transformless renderable entity found ({}), rendering at origin",
                            id
                        );
                        Mat4::IDENTITY
                    };

                    log::trace!(
                        "Pushing renderable mesh {:#?} with material {:#?} and transform {}",
                        mesh,
                        material,
                        transform
                    );

                    Renderable {
                        mesh,
                        material,
                        object_to_world: transform,
                    }
                })
        }
    }

    fn exec_engine_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::AddSystem(system) => self.systems.push(system),
            EngineCommand::OpenWindow(params) => self
                .eventloop
                .send_event(WindowingEvent::OpenWindow(params))
                .unwrap(),
            EngineCommand::SpawnEntity(components) => {
                let new_entity = self.world.create_entity();
                self.world.add_components_to_entity(new_entity, components);
            }
            EngineCommand::DestroyEntity(id) => {
                self.world.remove_entity(id);
            }
        }
    }

    fn exec_engine_commands(&mut self, commands: impl IntoIterator<Item = EngineCommand>) {
        for command in commands {
            self.exec_engine_command(command);
        }
    }

    fn run_systems_for_phase(&mut self, phase: SystemPhase) {
        let mut commands = Command::empty();

        for system in self.systems.iter_mut().filter(|sys| sys.phase == phase) {
            let ret = (system.func)(&mut self.world);
            commands.merge_with(ret);
        }

        self.exec_engine_commands(commands.consume());
    }

    fn for_each_plugin_mut(
        &mut self,
        mut func: impl FnMut(&mut Command, &mut Box<dyn WutEnginePlugin>),
    ) {
        let mut commands = Command::empty();

        for plugin in &mut self.plugins {
            func(&mut commands, plugin);
        }

        self.exec_engine_commands(commands.consume());
    }

    fn start(&mut self) {
        self.for_each_plugin_mut(|commands, plugin| plugin.on_start(commands));

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

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.started {
            log::trace!("about_to_wait fired but engine not yet initialized");
            return;
        }

        log::trace!("Starting frame");

        self.run_systems_for_phase(SystemPhase::Update);

        let renderables = unsafe { self.get_renderables() };

        unsafe {
            let contexts = self
                .world
                .query(|_id, args: (&Camera, Option<&Transform>)| {
                    let (camera, transform) = args;

                    let window = match self.windows.get(&camera.display) {
                        Some(window) => window,
                        None => {
                            log::warn!(
                                "Camera trying to render to non-existing window {}",
                                &camera.display
                            );

                            return None;
                        }
                    };

                    let view_mat = transform
                        .map(|t| t.local_to_world())
                        .unwrap_or(Mat4::IDENTITY);
                    let window_size: (u32, u32) = window.inner_size().into();

                    Some(camera.to_context(view_mat, window_size))
                });

            for context in contexts.into_iter().flatten() {
                self.renderer.render(context, &renderables);
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WindowingEvent) {
        log::debug!("Handling WutEngine WindowingEvent:\n{:#?}", event);

        match event {
            WindowingEvent::OpenWindow(params) => {
                if self.windows.contains_key(&params.id) && !params.ignore_existing {
                    panic!("Window {} already exists!", params.id);
                }

                let attrs = Window::default_attributes()
                    .with_title(params.title)
                    .with_min_inner_size(PhysicalSize::<u32>::from((640u32, 480u32)))
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

        self.for_each_plugin_mut(|commands, plugin| {
            plugin.on_window_event(&identifier, &event, commands)
        });

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

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.for_each_plugin_mut(|commands, plugin| {
            plugin.on_device_event(device_id, &event, commands)
        });
    }
}
