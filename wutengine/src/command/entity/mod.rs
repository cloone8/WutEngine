use super::Command;

mod builder;

pub use builder::*;

#[repr(transparent)]
pub struct EntityCommand<'a> {
    cmd: &'a mut Command,
}

impl<'a> EntityCommand<'a> {
    pub(crate) fn new(cmd: &'a mut Command) -> Self {
        Self { cmd }
    }
}

impl<'a> EntityCommand<'a> {
    pub fn spawn(&'a mut self) -> EntityBuilder<'a> {
        EntityBuilder::new(self)
    }
}
