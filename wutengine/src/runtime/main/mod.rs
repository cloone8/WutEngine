use std::collections::HashMap;

use rayon::prelude::*;

use wutengine_core::identifiers::WindowIdentifier;
use wutengine_graphics::renderer::WutEngineRenderer;

use crate::component::data::{ComponentData, ComponentState};
use crate::component::{self};
use crate::context::{
    EngineContext, GameObjectContext, GraphicsContext, MessageContext, PluginContext,
    ViewportContext, WindowContext,
};
use crate::gameobject::GameObjectId;
use crate::plugins::{self, WutEnginePlugin};
use crate::windowing::window::{Window, WindowData};

use super::{MessageQueue, Runtime, WindowingEvent};

mod lifecycle_hooks;
mod winit_loop;

impl<R: WutEngineRenderer> Runtime<R> {
    fn start(&mut self) {
        self.run_plugin_hooks(|plugin, context| {
            plugin.on_start(context);
        });

        self.started = true;
    }

    fn run_component_hook_on_active(
        &mut self,
        func: impl Fn(&mut ComponentData, &mut component::Context) + Send + Sync,
    ) {
        self.run_component_hook(|component| component.state == ComponentState::Active, func);
    }

    fn run_component_hook(
        &mut self,
        filter: impl Fn(&ComponentData) -> bool + Sync,
        func: impl Fn(&mut ComponentData, &mut component::Context) + Send + Sync,
    ) {
        let message_queue = MessageQueue::new();

        // Run the main user-provided hook
        self.run_component_func_with_context(
            &message_queue,
            filter,
            |_| (),
            |_, comp, context| func(comp, context),
        );

        // Now handle any messages that resulted from the calls to the hook
        self.run_message_queue(message_queue);
    }

    fn run_component_func_with_context<F, Fi, Fm, M>(
        &mut self,
        message_queue: &MessageQueue,
        component_filter: Fi,
        meta_func: Fm,
        func: F,
    ) where
        Fi: Fn(&ComponentData) -> bool + Sync,
        Fm: Fn(GameObjectId) -> M + Send + Sync,
        F: Fn(&M, &mut ComponentData, &mut component::Context) + Send + Sync,
    {
        let engine_context = EngineContext::new();
        let message_context = MessageContext::new(message_queue);
        let plugin_context = PluginContext::new(&self.plugins);
        let viewport_context = ViewportContext::new();
        let graphics_context = GraphicsContext::new();

        let window_data = make_windowdata_map(&self.windows);
        let window_context = WindowContext::new(&window_data);

        self.objects.par_iter_mut().for_each(|gameobject| {
            let meta = meta_func(gameobject.id);

            let mut cur_components = gameobject.components.borrow_mut();
            let mut new_components = Vec::new();

            for i in 0..cur_components.len() {
                if !component_filter(&cur_components[i]) {
                    continue;
                }

                let (component, go_context) =
                    GameObjectContext::new(gameobject, &mut cur_components, i);

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

            cur_components.extend(new_components.into_iter().map(ComponentData::new));
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

        let window_data = make_windowdata_map(&self.windows);

        let mut context = plugins::Context::new(&window_data, &message_queue);

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
                |_| true,
                |gameobject_id| {
                    let mut messages_for_gameobject = Vec::new();
                    message_queue.get_messages_for(gameobject_id, &mut messages_for_gameobject);

                    messages_for_gameobject
                },
                |messages, component_data, context| {
                    for message in messages {
                        component_data.component.on_message(context, message);
                    }
                },
            );
            // Replace the old and handled queue with the new one
            message_queue = new_queue;

            message_iter += 1;
        }
    }
}

fn make_windowdata_map(
    window_map: &HashMap<WindowIdentifier, Window>,
) -> HashMap<WindowIdentifier, WindowData> {
    let mut new_map = HashMap::with_capacity(window_map.len());

    for (k, v) in window_map.iter() {
        new_map.insert(k.clone(), v.window_data.clone());
    }

    new_map
}
