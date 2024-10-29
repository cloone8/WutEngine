//! The main runtime and its main loop.

use std::collections::HashMap;

use messaging::MessageQueue;
use rayon::prelude::*;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use winit::{application::ApplicationHandler, dpi::PhysicalSize};
use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::component::Component;
use crate::context::{
    EngineContext, GameObjectContext, MessageContext, ViewportContext, WindowContext,
};
use crate::context::{GraphicsContext, PluginContext};
use crate::gameobject::{GameObject, GameObjectId};
use crate::plugins::{self, WutEnginePlugin};
use crate::renderer::queue::RenderQueue;
use crate::time::Time;
use crate::windowing::WindowingEvent;
use crate::{component, windowing};

mod init;
pub mod messaging;
mod threadpool;

pub use init::*;

/// The main runtime for WutEngine. Cannot be constructed directly. Instead,
/// construct a runtime with a [RuntimeInitializer]
/// TODO: Split up runtime object into multiple smaller structs
/// for cleaner code
pub struct Runtime<R: WutEngineRenderer> {
    identmap: HashMap<GameObjectId, usize>,
    objects: Vec<GameObject>,

    render_queue: RenderQueue,

    eventloop: EventLoopProxy<WindowingEvent>,

    window_id_map: HashMap<WindowId, WindowIdentifier>,
    windows: HashMap<WindowIdentifier, Window>,

    started: bool,

    plugins: Vec<Box<dyn WutEnginePlugin>>,
    renderer: R,
}

impl<R: WutEngineRenderer> Runtime<R> {
    fn start(&mut self) {
        self.run_plugin_hooks(|plugin, context| {
            plugin.on_start(context);
        });

        self.started = true;
    }

    fn run_component_hook(
        &mut self,
        func: impl Fn(&mut Box<dyn Component>, &mut component::Context) + Send + Sync,
    ) {
        let message_queue = MessageQueue::new();

        // Run the main user-provided hook
        self.run_component_func_with_context(
            &message_queue,
            |_| (),
            |_, comp, context| func(comp, context),
        );

        // Now handle any messages that resulted from the calls to the hook
        self.run_message_queue(message_queue);
    }

    fn run_component_func_with_context<F, Fm, M>(
        &mut self,
        message_queue: &MessageQueue,
        meta_func: Fm,
        func: F,
    ) where
        Fm: Fn(GameObjectId) -> M + Send + Sync,
        F: Fn(&M, &mut Box<dyn Component>, &mut component::Context) + Send + Sync,
    {
        let engine_context = EngineContext::new();
        let message_context = MessageContext::new(message_queue);
        let plugin_context = PluginContext::new(&self.plugins);
        let viewport_context = ViewportContext::new();
        let graphics_context = GraphicsContext::new();
        let window_context = WindowContext::new(&self.windows);

        self.objects.par_iter_mut().for_each(|gameobject| {
            let meta = meta_func(gameobject.id);

            let mut new_components = Vec::new();

            for i in 0..gameobject.components.len() {
                let (component, go_context) = GameObjectContext::new(gameobject, i);

                let mut context = component::Context {
                    gameobject: go_context,
                    message: &message_context,
                    engine: &engine_context,
                    plugin: &plugin_context,
                    viewport: &viewport_context,
                    graphics: &graphics_context,
                    window: &window_context,
                };

                func(&meta, component, &mut context);

                new_components.extend(context.gameobject.consume());
            }

            gameobject.components.extend(new_components);
        });

        for new_gameobject in engine_context.consume() {
            match self.identmap.contains_key(&new_gameobject.id) {
                true => log::error!(
                    "Tried to add an already existing GameObject, ignoring : {}",
                    new_gameobject.id
                ),
                false => {
                    let go_id = new_gameobject.id;
                    let new_idx = self.objects.len();

                    self.identmap.insert(go_id, new_idx);
                    self.objects.push(new_gameobject);

                    log::debug!(
                        "Added new GameObject with ID {} at index {}",
                        go_id,
                        new_idx
                    );
                }
            }
        }

        self.render_queue.add_viewports(viewport_context);
        self.render_queue.add_renderables(graphics_context);

        for new_window_params in window_context.consume() {
            self.eventloop
                .send_event(WindowingEvent::OpenWindow(new_window_params))
                .unwrap();
        }
    }

    fn run_plugin_hooks(
        &mut self,
        func: impl Fn(&mut Box<dyn WutEnginePlugin>, &mut plugins::Context),
    ) {
        let message_queue = MessageQueue::new();
        let mut context = plugins::Context::new(&self.windows, &message_queue);

        for plugin in &mut self.plugins {
            func(plugin, &mut context);
        }

        let engine_context = context.engine;
        let viewport_context = context.viewport;
        let graphics_context = context.graphics;
        let window_context = context.windows;

        for new_gameobject in engine_context.consume() {
            match self.identmap.contains_key(&new_gameobject.id) {
                true => log::error!(
                    "Tried to add an already existing GameObject, ignoring : {}",
                    new_gameobject.id
                ),
                false => {
                    let go_id = new_gameobject.id;
                    let new_idx = self.objects.len();

                    self.identmap.insert(go_id, new_idx);
                    self.objects.push(new_gameobject);

                    log::debug!(
                        "Added new GameObject with ID {} at index {}",
                        go_id,
                        new_idx
                    );
                }
            }
        }

        self.render_queue.add_viewports(viewport_context);
        self.render_queue.add_renderables(graphics_context);

        for new_window_params in window_context.consume() {
            self.eventloop
                .send_event(WindowingEvent::OpenWindow(new_window_params))
                .unwrap();
        }

        self.run_message_queue(message_queue);
    }

    /// Runs the given message queue, calling the appropriate callbacks to handle each message.
    /// If any new messages are sent while handling the original messages, these are handled too.
    /// This repeats until no more messages are sent.
    fn run_message_queue(&mut self, mut message_queue: MessageQueue) {
        let mut message_iter = 0usize;

        loop {
            // No more messages, done!
            if message_queue.is_empty() {
                return;
            }

            log::trace!("Message handling loop, iteration {}", message_iter);

            let new_queue = MessageQueue::new();

            self.run_component_func_with_context(
                &new_queue,
                |gameobject_id| {
                    let mut messages_for_gameobject = Vec::new();
                    message_queue.get_messages_for(gameobject_id, &mut messages_for_gameobject);

                    messages_for_gameobject
                },
                |messages, component, context| {
                    for message in messages {
                        component.on_message(context, message);
                    }
                },
            );
            // Replace the old and handled queue with the new one
            message_queue = new_queue;

            message_iter += 1;
        }
    }
}

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
