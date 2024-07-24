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
    pub fn open(&mut self, params: OpenWindowParams) {
        self.cmd
            .commands
            .push(crate::EngineCommand::OpenWindow(params));
    }
}

#[derive(Debug)]
pub struct OpenWindowParams {
    pub id: WindowIdentifier,
    pub title: String,
    pub ignore_existing: bool,
}
