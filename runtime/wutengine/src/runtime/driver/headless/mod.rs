//! Headless driver. Only runs the frame loop without any renderer interaction

use core::time::Duration;

use crate::runtime::frame::FrameState;

#[derive(Debug)]
pub(crate) struct HeadlessApp {
    frame_state: FrameState,
}

impl HeadlessApp {
    pub(crate) fn new() -> Self {
        Self {
            frame_state: FrameState::new(Duration::from_secs_f64(1.0 / 60.0)), //TODO: Make configurable
        }
    }

    pub(crate) fn run(&mut self) {}
}
