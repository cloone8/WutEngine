pub use glam as math;

pub struct Scene {
    name: String,
    entities: Vec<Entity>
}

pub struct Entity {
    name: String,
    controllers: Vec<Box<dyn EntityController>>,
    children: Vec<Entity>
}

pub trait EntityController {
    fn init(&mut self, entity: &mut Entity) { }

    fn tick(&mut self, entity: &mut Entity, delta_time: f32) { }
}
