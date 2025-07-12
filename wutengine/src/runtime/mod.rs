//! Main WutEngine runtime

use std::sync::Mutex;

use crate::{component, gameobject, time};

/// Runs a single frame
pub(crate) fn frame() {
    profiling::finish_frame!();
    profiling::function_scope!();

    log::trace!("Starting new frame");

    //TODO: Make time management better
    static FIXED_ACCUM: Mutex<f32> = Mutex::new(0.0);

    let fixed_timestep = time::Time::get().fixed_delta;

    unsafe {
        time::Time::update_to_now(fixed_timestep);
    }

    let mut time_locked = FIXED_ACCUM.lock().unwrap();

    *time_locked += time::Time::get().delta;

    //TODO END ^

    run_frame_phase("Update", || {
        component::run_on_active_components(|component, context| {
            component.on_update(context);
        });
    });

    {
        profiling::scope!("Fixed updates");

        while *time_locked >= fixed_timestep {
            run_frame_phase("Fixed update", || {
                component::run_on_active_components(|component, context| {
                    component.on_fixed_update(context);
                });
            });

            *time_locked -= fixed_timestep;
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
