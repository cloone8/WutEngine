//! The main frame/step functions

use crate::log;
use crate::time;

pub(super) fn run_frame() {
    profiling::function_scope!();

    time::begin_frame();
}
