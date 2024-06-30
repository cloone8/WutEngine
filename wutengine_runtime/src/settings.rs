#[derive(Debug, Clone)]
pub struct Settings {
    pub open_window: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { open_window: true }
    }
}
