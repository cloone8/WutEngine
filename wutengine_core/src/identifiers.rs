//! Shared identifier types used throughout the various WutEngine crates

use core::fmt::Display;

/// The unique identifier for a window. Used to identify
/// a unique OS window in a cross-platform way, with user-identifiable names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowIdentifier {
    id: String,
}

impl WindowIdentifier {
    /// Creates a new [WindowIdentifier]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl Display for WindowIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}
