use crate::GameState;

use super::game_entity::GameEntity;

pub trait EntityComponent {
    fn update(&self, delta_time: f32, this_entity: &GameEntity, state: &mut GameState);
    fn fixed_update(&self, fixed_delta_time: f32, this_entity: &GameEntity, state: &mut GameState);
}
