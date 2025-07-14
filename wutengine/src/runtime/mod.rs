//! Main WutEngine runtime

use crate::{component, gameobject, graphics, time};

/// Runs a single frame
pub(crate) fn run_step() {
    profiling::finish_frame!();
    profiling::function_scope!();

    log::trace!("Starting new frame");

    let fixed_updates = unsafe { time::update_to_now() };

    run_frame_phase("Update", || {
        component::run_on_active_components(|component, context| {
            component.on_update(context);
        });
    });

    {
        profiling::scope!("Fixed updates");

        for _ in 0..fixed_updates {
            run_frame_phase("Fixed update", || {
                component::run_on_active_components(|component, context| {
                    component.on_fixed_update(context);
                });
            });
        }
    }
}

fn run_frame_phase(_name: &'static str, phase: impl FnOnce()) {
    profiling::scope!(_name);

    component::add_queued();

    gameobject::handle_state_changes();
    component::handle_enable_disable();

    phase();

    component::handle_destruction();
    gameobject::cleanup_destroyed();
}

#[profiling::function]
pub(crate) fn render() {
    log::trace!("Rendering frame");

    graphics::render_all_windows();
}
