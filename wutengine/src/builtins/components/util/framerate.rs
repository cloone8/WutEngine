use std::time::Instant;

use wutengine_core::{Component, EntityId};
use wutengine_macro::system;

use crate::command::Command;
use crate::log;

#[derive(Debug, Default)]
pub struct FramerateCounter {
    prev_time: Option<Instant>,
    num_frames: usize,
}

impl FramerateCounter {
    pub fn new() -> Self {
        Self {
            prev_time: None,
            num_frames: 0,
        }
    }
}

impl Component for FramerateCounter {}

#[system(root_wutengine_crate = crate)]
pub fn framerate_counter_system(
    _commands: &mut Command,
    _entity: EntityId,
    counter: &mut FramerateCounter,
) {
    let cur_time = Instant::now();

    if let Some(prev_time) = &mut counter.prev_time {
        let diff = cur_time.duration_since(*prev_time);
        let as_secs = diff.as_secs_f64();
        let as_fps = 1.0 / as_secs;

        if counter.num_frames % 1000 == 0 {
            log::info!("Frametime: {}s - FPS: {}", as_secs, as_fps);
        }

        *prev_time = cur_time;
    } else {
        counter.prev_time = Some(cur_time);
    }

    counter.num_frames = counter.num_frames.wrapping_add(1);
}
