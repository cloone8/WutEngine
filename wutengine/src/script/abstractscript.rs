use super::{frame::Frame, start::Start};
use std::fmt::Debug;

pub trait AbstractScript: Debug {
    fn as_start(&mut self) -> Option<&mut dyn Start> {
        None
    }

    fn as_frame(&mut self) -> Option<&mut dyn Frame> {
        None
    }
}

impl AbstractScript for Box<dyn AbstractScript> {
    fn as_start(&mut self) -> Option<&mut dyn Start> {
        self.as_mut().as_start()
    }

    fn as_frame(&mut self) -> Option<&mut dyn Frame> {
        self.as_mut().as_frame()
    }
}
