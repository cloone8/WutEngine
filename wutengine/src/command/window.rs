use winit::window::Fullscreen;

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
    pub mode: FullscreenType,
    pub ignore_existing: bool,
}

#[derive(Debug)]
pub enum FullscreenType {
    Windowed,
    BorderlessFullscreenWindow,
    ExclusiveFullscreen,
}

impl From<FullscreenType> for Option<Fullscreen> {
    fn from(value: FullscreenType) -> Self {
        match value {
            FullscreenType::Windowed => None,
            FullscreenType::BorderlessFullscreenWindow => {
                todo!("Borderless fullscreen not yet implemented")
            }
            FullscreenType::ExclusiveFullscreen => {
                todo!("Exclusive fullscreen not yet implemented")
            }
        }
    }
}
