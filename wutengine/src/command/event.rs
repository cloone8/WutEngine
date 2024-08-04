use super::Command;

#[repr(transparent)]
pub struct EventCommand<'a> {
    cmd: &'a mut Command,
}

impl<'a> EventCommand<'a> {
    pub(crate) fn new(cmd: &'a mut Command) -> Self {
        Self { cmd }
    }
}
