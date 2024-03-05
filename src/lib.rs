use core::game_entity::GameEntity;
use std::time::Instant;

pub use glam;

pub mod core;

pub struct WutEngine {
    fixed_timestep: std::time::Duration,
    entities: Vec<GameEntity>,
}

pub struct GameState<'a, 'b> {
    engine: &'a WutEngine,
    update_stack: Vec<&'b GameEntity>,
    delta: StateDelta,
}

struct StateDelta {

}

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(engine: &WutEngine) -> GameState {
        GameState {
            engine,
            update_stack: Vec::new(),
            delta: StateDelta {},
        }
    }

    pub(crate) fn push_entity(&mut self, entity: &'b GameEntity) {
        self.update_stack.push(entity);
    }

    pub(crate) fn pop_entity(&mut self) {
        self.update_stack.pop();
    }

    pub(crate) fn get_update_stack(&self) -> &Vec<&'b GameEntity> {
        &self.update_stack
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

    fn do_update_for_entity<'a>(&self, entity: &'a GameEntity, state: &mut GameState<'_, 'a>, delta_time: f32) {
        println!("Update for entity: {}", entity.name());

        for component in entity.get_components() {
            component.update(delta_time, entity, state);
        }

        if entity.get_children().len() > 0 {
            state.push_entity(entity);
        }

        for child in entity.get_children() {
            self.do_update_for_entity(&child, state, delta_time);
        }

        if entity.get_children().len() > 0 {
            state.pop_entity();
        }
    }

    fn do_update(&self, delta_time: f32) -> StateDelta {
        println!("Update!");

        let mut state = GameState::new(self);

        for entity in &self.entities {
            self.do_update_for_entity(entity, &mut state, delta_time);
        }

        state.delta
    }

    fn do_fixed_update_for_entity<'a>(&self, entity: &'a GameEntity, state: &mut GameState<'_, 'a>, fixed_delta_time: f32) {
        println!("Fixed update for entity: {}", entity.name());

        for component in entity.get_components() {
            component.fixed_update(fixed_delta_time, entity, state);
        }

        if entity.get_children().len() > 0 {
            state.push_entity(entity);
        }

        for child in entity.get_children() {
            self.do_fixed_update_for_entity(&child, state, fixed_delta_time);
        }

        if entity.get_children().len() > 0 {
            state.pop_entity();
        }
    }

    fn do_fixed_update(&self, fixed_delta_time: f32) -> StateDelta {
        println!("Fixed update!");

        let mut state = GameState::new(self);

        for entity in &self.entities {
            self.do_fixed_update_for_entity(entity, &mut state, fixed_delta_time)
        }

        state.delta
    }

    fn do_render(&self) {
        println!("Render!");
        // TODO: Setup

        // TODO: Destroy
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

            // Fixed timestep loop
            while fixed_time_accumulator >= self.fixed_timestep.as_secs_f32() {
                // Put in seperate variable to avoid changes in fixed timestep to mess up the loop
                let curr_timestep = self.fixed_timestep.as_secs_f32();

                let state_delta = self.do_fixed_update(curr_timestep);

                fixed_time_accumulator -= curr_timestep;

                self.process_state_changes(state_delta);
            }

            // Per-frame update now
            let state_delta = self.do_update(delta_time);
            self.process_state_changes(state_delta);

            // Render here
            self.do_render();

        }
    }
}
