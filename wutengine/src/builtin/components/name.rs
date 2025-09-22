use crate::prelude::Component;

#[derive(Debug)]
pub struct Name(pub(crate) String);

impl Component for Name {}

impl Name {
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }
}
