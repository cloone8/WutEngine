use super::AbstractScript;

pub trait Frame: AbstractScript {
    fn on_frame(&mut self);
}
