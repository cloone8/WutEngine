use std::collections::HashMap;
use std::sync::Mutex;

use wutengine_core::identifiers::WindowIdentifier;

use crate::windowing::OpenWindowParams;
use crate::windowing::window::WindowData;

/// The window context, used for interacting with window related APIs.
#[must_use = "The commands within the context must be consumed"]
#[derive(Debug)]
pub struct WindowContext<'a> {
    pub(crate) windows: &'a HashMap<WindowIdentifier, WindowData>,
    opened: Mutex<Vec<OpenWindowParams>>,
}

impl<'a> WindowContext<'a> {
    /// Creates a new [WindowContext] with the given existing windows.
    pub(crate) fn new(windows: &'a HashMap<WindowIdentifier, WindowData>) -> Self {
        WindowContext {
            windows,
            opened: Mutex::new(Vec::new()),
        }
    }

    /// Consumes the context, returning the window commands within
    pub(crate) fn consume(self) -> Vec<OpenWindowParams> {
        self.opened.into_inner().unwrap()
    }

    /// Returns a reference to the given window, if it exists.
    pub fn get(&self, window: &WindowIdentifier) -> Option<&WindowData> {
        self.windows.get(window)
    }

    /// Submits a command to open a new window
    pub fn open(&self, params: OpenWindowParams) {
        let mut locked = self.opened.lock().unwrap();

        locked.push(params);
    }
}
