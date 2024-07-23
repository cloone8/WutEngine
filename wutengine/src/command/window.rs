use crate::core::windowing::WindowIdentifier;

use super::Command;

#[repr(transparent)]
pub struct WindowCommand<'a> {
    cmd: &'a mut Command,
}

impl<'a> WindowCommand<'a> {
    pub(crate) fn new(cmd: &'a mut Command) -> Self {
        Self { cmd }
    }
}

impl<'a> WindowCommand<'a> {
    pub fn open(&mut self, id: WindowIdentifier) {
        self.cmd.commands.push(crate::EngineCommand::OpenWindow(id));
    }
}
