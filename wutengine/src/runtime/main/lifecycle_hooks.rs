use wutengine_graphics::renderer::WutEngineRenderer;

use crate::gameobject;
use crate::runtime::Runtime;
use crate::runtime::main::ComponentState;

#[profiling::all_functions]
impl<R: WutEngineRenderer> Runtime<R> {
    pub(super) fn lifecycle_start(&mut self) {
        log::trace!("Starting new components");

        self.run_component_hook(
            |component| matches!(component.state, ComponentState::ReadyForStart),
            |component_data, context| {
                component_data.component.on_start(context);
                component_data.state = ComponentState::Active;
            },
        );
    }

    pub(super) fn lifecycle_destroy(&mut self) {
        log::trace!("Destroying dying components");

        self.run_component_hook(
            |component| matches!(component.state, ComponentState::Dying),
            |component_data, context| {
                component_data.component.on_destroy(context);
            },
        );

        gameobject::internal::with_storage_mut(|storage| {
            for go in &mut storage.objects {
                go.remove_dying_components();
            }
        });
    }

    pub(super) fn lifecycle_physics_update(&mut self) {
        log::trace!("Running physics update for plugins");

        self.run_plugin_hooks(|plugin, context| plugin.physics_update(context));

        log::trace!("Running physics update for components");

        self.run_component_hook_on_active(|component_data, context| {
            component_data.component.physics_update(context);
        });
    }

    pub(super) fn lifecycle_post_physics_update(&mut self) {
        log::trace!("Running post-physics update for plugins");

        self.run_plugin_hooks(|plugin, context| plugin.post_physics_update(context));

        log::trace!("Running post-physics update for components");

        self.run_component_hook_on_active(|component_data, context| {
            component_data.component.post_physics_update(context);
        });
    }

    pub(super) fn lifecycle_physics_solver_update(&mut self) {
        log::trace!("Running physics solver update for plugins");

        self.run_plugin_hooks(|plugin, context| plugin.physics_solver_update(context));
    }

    pub(super) fn lifecycle_pre_update(&mut self) {
        log::trace!("Running pre-update for plugins");

        self.run_plugin_hooks(|plugin, context| plugin.pre_update(context));

        log::trace!("Running pre-update for components");

        self.run_component_hook_on_active(|component_data, context| {
            component_data.component.pre_update(context);
        });
    }

    pub(super) fn lifecycle_update(&mut self) {
        log::trace!("Running update for plugins");

        self.run_plugin_hooks(|plugin, context| plugin.update(context));

        log::trace!("Running update for components");

        self.run_component_hook_on_active(|component_data, context| {
            component_data.component.update(context);
        });
    }

    pub(super) fn lifecycle_pre_render(&mut self) {
        log::trace!("Running pre-render for components");

        self.run_component_hook_on_active(|component_data, context| {
            component_data.component.pre_render(context);
        });
    }
}
