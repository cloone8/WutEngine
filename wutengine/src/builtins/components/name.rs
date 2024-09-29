use wutengine_core::Component;

/// A simple name component for an entity.
#[derive(Debug)]
pub struct Name(pub String);

impl Name {
    /// Creates a new [Name] component
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }
}

impl Component for Name {}
