//! The main per-frame render functionality

use std::sync::Mutex;

use wutengine_util::GlobalManager;

use crate::prelude::{Camera, Name};
use crate::runtime::world::WORLD_MANAGER;

pub trait RenderPass: Send + 'static {
    fn name() -> &'static str
    where
        Self: Sized;

    fn execute(&mut self, target: &Camera);
}

pub(crate) struct RenderPipeline {
    pipeline: Mutex<Vec<RenderPassInfo>>,
}

impl RenderPipeline {
    fn new() -> Self {
        Self {
            pipeline: Mutex::new(Vec::new()),
        }
    }
}

pub(crate) static RENDER_PIPELINE_MANAGER: GlobalManager<RenderPipeline> = GlobalManager::new();

/// Initializes the global render pipeline manager
pub(crate) fn init() {
    GlobalManager::init(&RENDER_PIPELINE_MANAGER, RenderPipeline::new());
}

struct RenderPassInfo {
    name: &'static str,
    pass: Box<dyn RenderPass>,
}

pub fn insert_render_pass<P: RenderPass>(pass: P) {
    log::debug!("Inserting new renderpass: {}", P::name());

    RENDER_PIPELINE_MANAGER
        .pipeline
        .lock()
        .unwrap()
        .push(RenderPassInfo {
            name: P::name(),
            pass: Box::new(pass),
        })
}

fn render_camera(camera: &Camera, name: &str, passes: &mut [RenderPassInfo]) {
    profiling::function_scope!(name);

    log::debug!("Rendering camera: {name}");

    for pass in passes.iter_mut() {
        profiling::scope!(pass.name);

        log::debug!("Rendering pass: {}", pass.name);

        pass.pass.execute(camera);
    }
}

#[profiling::function]
pub(crate) fn render_frame() {
    log::debug!("Rendering frame");
    let world = WORLD_MANAGER.shared();

    let mut passes = RENDER_PIPELINE_MANAGER.pipeline.lock().unwrap();
    let mut cam_query = world.query::<(&Camera, Option<&Name>)>();

    for (_, (camera, name)) in cam_query.iter() {
        let cam_name = match name {
            Some(name_component) => name_component.0.as_str(),
            None => "(unnamed)",
        };

        render_camera(camera, cam_name, &mut passes);
    }
}
