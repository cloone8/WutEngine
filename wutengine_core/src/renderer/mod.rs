use core::{fmt::Display, hash::Hash};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use renderable::Renderable;

pub mod renderable;

pub trait WutEngineRenderer: Sized {
    const NAME: &'static str;

    fn init() -> Self;
    fn init_window(&mut self, id: WindowId, handles: WindowHandles, viewport: (u32, u32));
    fn render(&mut self, window: WindowId, objects: &[Renderable]);
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowId(u64);

impl WindowId {
    pub fn new(raw: u64) -> Self {
        WindowId(raw)
    }
}

impl Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Note: Implemented manually because [nohash_hasher::IsEnabled] requires that
/// implementors guarantee that only a single `write_*` function is ever called
/// on the hasher
impl Hash for WindowId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl nohash_hasher::IsEnabled for WindowId {}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct WindowHandles<'a> {
    pub window: WindowHandle<'a>,
    pub display: DisplayHandle<'a>,
}

impl<'a> WindowHandles<'a> {
    pub fn from_window<W>(window: &'a W) -> Result<Self, HandleError>
    where
        W: HasWindowHandle + HasDisplayHandle,
    {
        Ok(Self {
            window: window.window_handle()?,
            display: window.display_handle()?,
        })
    }
}
