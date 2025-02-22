use wutengine::component::Component;
use wutengine::macros::component_boilerplate;

#[derive(Debug)]
pub struct Enemy;

impl Component for Enemy {
    component_boilerplate!();
}
