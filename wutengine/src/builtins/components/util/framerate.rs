use core::num::Saturating;
use std::time::Instant;

use wutengine_core::{Component, EntityId};
use wutengine_macro::system;

use crate::command::Command;
use crate::log;

#[derive(Debug)]
pub struct FramerateCounter {
    prev_time: Instant,
    num_frames: Saturating<usize>,
}

impl FramerateCounter {
    pub fn new() -> Self {
        Self {
            prev_time: Instant::now(),
            num_frames: Saturating(0),
        }
    }
}

impl Default for FramerateCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for FramerateCounter {}

#[system(root_wutengine_crate = crate)]
pub fn framerate_counter_system(
    _commands: &mut Command,
    _entity: EntityId,
    counter: &mut FramerateCounter,
) {
    counter.num_frames += 1;

    let cur_time = Instant::now();

    let duration_since_last_report = cur_time.duration_since(counter.prev_time).as_secs_f64();

    if duration_since_last_report >= 1.0 {
        let fps = counter.num_frames.0 as f64 / duration_since_last_report;

        log::info!("FPS: {}, avg. frametime: {}ms", fps, (1.0 / fps) * 1000.0);

        counter.num_frames = Saturating(0);
        counter.prev_time = cur_time;
    }
}
