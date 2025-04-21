use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use wutengine_graphics::renderer::{Renderable, WutEngineRenderer};

use crate::builtins::assets::{RawMaterial, RawMesh};
use crate::runtime::main::ComponentState;
use crate::runtime::{EXIT_REQUESTED, Runtime};
use crate::time::Time;
use crate::windowing;
use crate::windowing::window::Window;

use super::WindowingEvent;

#[profiling::all_functions]
impl<R: WutEngineRenderer> ApplicationHandler<WindowingEvent> for Runtime<R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.started {
            log::info!("Initializing WutEngine");
            windowing::display::configure(event_loop);
            self.start();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        profiling::finish_frame!();

        if !self.started {
            log::trace!("about_to_wait fired but engine not yet initialized");
            return;
        }

        if EXIT_REQUESTED.load(core::sync::atomic::Ordering::SeqCst) {
            if !event_loop.exiting() {
                log::debug!("Exit requested. Notifying event loop");
                event_loop.exit();
            }
            log::trace!("Skipping frame due to exit in progress");
            return;
        }

        log::trace!("Updating current window information");
        {
            profiling::scope!("Update Windows");
            for window in self.windows.values_mut() {
                window.update();
            }
        }

        log::trace!("Starting new frame");

        unsafe {
            profiling::scope!("Update Time");
            Time::update_to_now(self.physics_update_interval);
        }

        self.lifecycle_start();

        // Physics usually runs at a fixed interval
        {
            profiling::scope!("Phyiscs Pipeline");

            let mut physics_steps = 0;
            if self.physics_update_interval == 0.0 {
                profiling::scope!("Physics Step");
                physics_steps = 1;

                log::trace!("Physics synced with framerate. Running iteration");

                // 0.0 means sync with frame
                self.lifecycle_physics_update();
                self.lifecycle_post_physics_update();
                self.lifecycle_physics_solver_update();
            } else {
                log::trace!("Physics running at interval. Running variable number of steps");

                // Any other value means "run at that fixed timestep"
                self.physics_update_accumulator += Time::get().delta;

                while self.physics_update_accumulator >= self.physics_update_interval {
                    profiling::scope!("Physics Step");
                    physics_steps += 1;

                    log::trace!("Running physics step");

                    self.lifecycle_physics_update();
                    self.lifecycle_post_physics_update();
                    self.lifecycle_physics_solver_update();
                    self.physics_update_accumulator -= self.physics_update_interval;
                }
            }
            log::trace!("Ran {} physics steps this frame", physics_steps);
        }

        self.lifecycle_pre_update();
        self.lifecycle_update();
        self.lifecycle_pre_render();

        log::trace!("Running component destructors");
        self.lifecycle_destroy();

        log::trace!("Doing rendering");

        {
            profiling::scope!("Rendering");

            let mut renderables = Vec::with_capacity(self.render_queue.renderables.len());

            {
                profiling::scope!("Resolve Render Commands");

                for render_command in &self.render_queue.renderables {
                    renderables.push(Renderable {
                        material: RawMaterial::flush_and_get_id(
                            &render_command.material,
                            &mut self.renderer,
                        ),
                        mesh: RawMesh::flush_and_get_id(&render_command.mesh, &mut self.renderer),
                        object_to_world: render_command.object_to_world,
                    });
                }
            }

            {
                profiling::scope!("Render Viewports");

                for viewport in &self.render_queue.viewports {
                    profiling::scope!("Render Single Viewport");
                    self.renderer.render(viewport, &renderables);
                }
            }

            self.render_queue.viewports.clear();
            self.render_queue.renderables.clear();
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WindowingEvent) {
        log::trace!("Handling WutEngine WindowingEvent:\n{:#?}", event);

        match event {
            WindowingEvent::OpenWindow(params) => {
                profiling::scope!("Open Window");

                if self.windows.contains_key(&params.id) && !params.ignore_existing {
                    panic!("Window {} already exists!", params.id);
                }

                let attrs = winit::window::Window::default_attributes()
                    .with_title(params.title)
                    .with_min_inner_size(PhysicalSize::<u32>::from((640u32, 480u32)))
                    .with_fullscreen(params.mode.into());

                let window = event_loop.create_window(attrs).unwrap();

                self.renderer
                    .new_window(&params.id, &window, window.inner_size().into());

                let old_val = self.window_id_map.insert(window.id(), params.id.clone());

                debug_assert!(old_val.is_none());

                let old_val = self.windows.insert(params.id, Window::new(window));

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

        self.run_plugin_hooks(|plugin, context| {
            plugin.on_window_event(&identifier, &event, context)
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

                self.renderer.window_size_changed(&identifier, size.into());

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
        self.run_plugin_hooks(|plugin, context| plugin.on_device_event(device_id, &event, context));
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        log::info!("WutEngine shutting down");

        // Mark all components as dying and cancel all components queued for startup.
        for go in &mut self.obj_storage.objects {
            go.cancel_component_creation();
            for component in go.components.get_mut() {
                component.state = ComponentState::Dying;
            }
        }

        // Run the destruction lifecycle hook
        self.lifecycle_destroy();

        // Final thing: flush all logs
        log::logger().flush();
    }
}
