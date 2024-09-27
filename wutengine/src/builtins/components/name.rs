use wutengine_core::Component;

#[derive(Debug)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }
}

impl Component for Name {}
