use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::runtime::Runtime;
use crate::time::Time;
use crate::windowing;
use crate::windowing::window::Window;

use super::WindowingEvent;

impl<R: WutEngineRenderer> ApplicationHandler<WindowingEvent> for Runtime<R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.started {
            log::info!("Initializing WutEngine");
            windowing::display::configure(event_loop);
            self.start();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if !self.started {
            log::trace!("about_to_wait fired but engine not yet initialized");
            return;
        }

        log::trace!("Updating current window information");
        for window in self.windows.values_mut() {
            window.update();
        }

        log::trace!("Starting new frame");

        unsafe {
            Time::update_to_now();
        }

        log::trace!("Running pre-update for plugins");

        self.run_plugin_hooks(|plugin, context| plugin.pre_update(context));

        log::trace!("Running pre-update for components");

        self.run_component_hook(|component, context| {
            component.pre_update(context);
        });

        log::trace!("Running update for components");

        self.run_component_hook(|component, context| {
            component.update(context);
        });

        log::trace!("Running pre-render for components");

        self.run_component_hook(|component, context| {
            component.pre_render(context);
        });

        log::trace!("Doing rendering");

        for viewport in &self.render_queue.viewports {
            self.renderer
                .render(viewport, &self.render_queue.renderables);
        }

        self.render_queue.viewports.clear();
        self.render_queue.renderables.clear();
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WindowingEvent) {
        log::trace!("Handling WutEngine WindowingEvent:\n{:#?}", event);

        match event {
            WindowingEvent::OpenWindow(params) => {
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
        self.run_plugin_hooks(|plugin, context| plugin.on_device_event(device_id, &event, context));
    }
}
