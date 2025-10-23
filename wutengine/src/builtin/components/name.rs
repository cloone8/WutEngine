use crate::prelude::Component;

/// A simple name component allowing for naming entities
#[derive(Debug, Clone)]
pub struct Name(pub(crate) String);

impl Component for Name {}

impl Name {
    /// Creates a new [Name] component
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }
}
