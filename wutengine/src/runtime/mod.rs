//! Main WutEngine runtime

use crate::{component, gameobject};

/// Runs a single frame
pub(crate) fn frame() {
    profiling::finish_frame!();

    log::trace!("Starting new frame");

    component::add_queued();

    gameobject::handle_state_changes();

    component::handle_enable_disable();
    component::run_update();
    component::handle_destruction();

    gameobject::cleanup_destroyed();
}
