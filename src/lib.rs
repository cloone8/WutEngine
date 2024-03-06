use core::game_entity::GameEntity;
use std::{any::Any, time::Instant};

pub use glam;
pub use winit;
use winit::{dpi::PhysicalSize, event_loop::{EventLoop, EventLoopBuilder}, window::WindowId};

pub mod core;
pub mod components;

pub trait EngineState {}
pub struct UninitializedState {
    windows: Vec<(String, PhysicalSize<u32>)>
}
pub struct InitializedState {
    event_loop: EventLoop<()>,
    windows: Vec<WindowId>
}

impl EngineState for UninitializedState { }
impl EngineState for InitializedState { }

pub struct WutEngine<S: EngineState = UninitializedState> {
    fixed_timestep: std::time::Duration,
    entities: Vec<GameEntity>,
    data: S
}

pub struct GameState<'a, 'b> {
    engine: &'a WutEngine<InitializedState>,
    update_stack: Vec<&'b GameEntity>,
    delta: StateDelta,
}

struct StateDelta;

impl<'a, 'b> GameState<'a, 'b> {
    pub fn new(engine: &WutEngine<InitializedState>) -> GameState {
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
    pub fn new() -> WutEngine<UninitializedState> {
        log::info!("Building WutEngine instance");

        WutEngine {
            fixed_timestep: std::time::Duration::from_secs_f32(0.02),
            entities: Vec::new(),
            data: UninitializedState {
                windows: Vec::new()
            },
        }
    }
}

impl WutEngine<UninitializedState> {
    pub fn add_entity(&mut self, entity: GameEntity) -> &mut Self {
        self.entities.push(entity);
        self
    }

    pub fn add_window(&mut self, initial_size: PhysicalSize<u32>, title: &str) -> &mut Self {
        log::info!("Adding window");

        self.data.windows.push((title.to_string(), initial_size));
        self
    }

    pub fn initialize(self) -> WutEngine<InitializedState> {
        log::info!("Initializing WutEngine instance");

        let event_loop = EventLoopBuilder::new().build().unwrap();
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let windows = self.data.windows.iter().map(|(title, size)| {
            winit::window::WindowBuilder::new()
                .with_title(title)
                .with_inner_size(*size)
                .build(&event_loop)
                .unwrap()
                .id()
        }).collect();

        log::info!("Initialization done");

        WutEngine {
            fixed_timestep: self.fixed_timestep,
            entities: self.entities,
            data: InitializedState {
                event_loop,
                windows
            },
        }
    }
}

impl WutEngine<InitializedState> {
    fn process_state_changes(&mut self, delta: StateDelta) {
        log::debug!("Processing state changes!");
    }

    fn do_update_for_entity<'a>(&self, entity: &'a GameEntity, state: &mut GameState<'_, 'a>, delta_time: f32) {
        log::trace!("Update for entity: {}", entity.name());

        for component in entity.get_components() {
            log::trace!("Update for component: {:?}", component.type_id());

            component.update(delta_time, entity, state);
        }

        if entity.get_children().len() > 0 {
            state.push_entity(entity);
        }

        log::trace!("Running update for {} children of entity {}", entity.get_children().len(), entity.name());

        for child in entity.get_children() {
            self.do_update_for_entity(&child, state, delta_time);
        }

        if entity.get_children().len() > 0 {
            state.pop_entity();
        }
    }

    fn do_update(&self, delta_time: f32) -> StateDelta {
        log::debug!("Running update");

        let mut state = GameState::new(self);

        for entity in &self.entities {
            self.do_update_for_entity(entity, &mut state, delta_time);
        }

        state.delta
    }

    fn do_fixed_update_for_entity<'a>(&self, entity: &'a GameEntity, state: &mut GameState<'_, 'a>, fixed_delta_time: f32) {
        log::trace!("Fixed update for entity: {}", entity.name());

        for component in entity.get_components() {
            log::trace!("Fixed update for component: {:?}", component.type_id());

            component.fixed_update(fixed_delta_time, entity, state);
        }

        if entity.get_children().len() > 0 {
            state.push_entity(entity);
        }

        log::trace!("Running fixedupdate for {} children of entity {}", entity.get_children().len(), entity.name());

        for child in entity.get_children() {
            self.do_fixed_update_for_entity(&child, state, fixed_delta_time);
        }

        if entity.get_children().len() > 0 {
            state.pop_entity();
        }
    }

    fn do_fixed_update(&self, fixed_delta_time: f32) -> StateDelta {
        log::debug!("Running fixed update");

        let mut state = GameState::new(self);

        for entity in &self.entities {
            self.do_fixed_update_for_entity(entity, &mut state, fixed_delta_time)
        }

        state.delta
    }

    fn do_render(&self) {
        log::debug!("Rendering frame");
        // TODO: Setup

        // TODO: Destroy
    }

    pub fn run(mut self) {
        log::info!("Running WutEngine instance");

        let mut time = Instant::now();
        let mut fixed_time_accumulator: f32 = 0.0;

        log::info!("Running event loop");

        self.data.event_loop.run(move |event, elwt| {
            log::trace!("Incoming eventloop event: {:?}", event);

        });

        loop {
            log::debug!("Frame start");

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

            log::debug!("Frame end");
        }
    }
}
