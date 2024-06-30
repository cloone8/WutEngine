use super::AbstractScript;

pub trait Start: AbstractScript {
    fn on_start(&mut self);
}
