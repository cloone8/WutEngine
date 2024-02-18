use core::game_entity::GameEntity;
use std::time::Instant;

pub use glam;

pub mod core;

pub struct WutEngine {
    fixed_timestep: std::time::Duration,
    entities: Vec<GameEntity>,
}

pub struct GameState<'a> {
    engine: &'a WutEngine,
    delta: StateDelta,
}

struct StateDelta {

}

impl GameState<'_> {
    pub fn new(engine: &WutEngine) -> GameState {
        GameState {
            engine,
            delta: StateDelta {},
        }
    }
}

impl WutEngine {
    pub fn new() -> WutEngine {
        WutEngine {
            fixed_timestep: std::time::Duration::from_secs_f32(0.02),
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: GameEntity) {
        self.entities.push(entity);
    }

    pub fn initialize(&self) {
        println!("WutEngine is initialized!");
    }

    fn process_state_changes(&mut self, delta: StateDelta) {
        println!("Processing state changes!");
    }

    fn do_update(&self, delta_time: f32) -> StateDelta {
        println!("Update!");

        let mut state = GameState::new(self);

        for entity in &self.entities {
            println!("Entity: {}", entity.name());

            for component in entity.get_components() {
                component.update(delta_time, &entity, &mut state);
            }
        }

        state.delta
    }

    fn do_fixed_update(&self, fixed_delta_time: f32) -> StateDelta {
        println!("Fixed update!");

        let mut state = GameState::new(self);

        for entity in &self.entities {
            println!("Entity: {}", entity.name());

            for component in entity.get_components() {
                component.fixed_update(fixed_delta_time, &entity, &mut state);
            }
        }

        state.delta
    }

    pub fn run(mut self) {
        println!("WutEngine is running!");

        let mut time = Instant::now();
        let mut fixed_time_accumulator: f32 = 0.0;

        loop {
            let frame_start_time = Instant::now();
            let delta_time = frame_start_time.duration_since(time).as_secs_f32();

            time = frame_start_time;
            fixed_time_accumulator += delta_time;

            while fixed_time_accumulator >= self.fixed_timestep.as_secs_f32() {
                // Put in seperate variable to avoid changes in fixed timestep to mess up the loop
                let curr_timestep = self.fixed_timestep.as_secs_f32();

                let state_delta = self.do_fixed_update(curr_timestep);

                fixed_time_accumulator -= curr_timestep;

                self.process_state_changes(state_delta);
            }

            let state_delta = self.do_update(delta_time);
            self.process_state_changes(state_delta);
        }
    }
}
