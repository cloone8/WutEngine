use core::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowIdentifier {
    id: String,
}

impl WindowIdentifier {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Display for WindowIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}
