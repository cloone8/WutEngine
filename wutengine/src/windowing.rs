#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WindowIdentifier {
    id: String,
}

impl WindowIdentifier {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}
