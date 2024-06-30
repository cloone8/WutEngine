pub trait WutEngineRenderer: Default {
    const NAME: &'static str;

    fn init(&mut self);
    fn render(&mut self, objects: &[Renderable]);
}

pub struct Renderable {}
