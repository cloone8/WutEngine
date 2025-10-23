//! The main per-frame render functionality

use std::sync::Mutex;

use wutengine_graphics::wgpu;
use wutengine_util::GlobalManager;

use crate::prelude::{Camera, Name};
use crate::runtime::world::WORLD_MANAGER;

/// Per-viewport info for a [RenderPass]
#[derive(Debug)]
pub struct ViewportInfo {}

/// A single rendering pass
pub trait RenderPass: Send + 'static {
    /// The (constant) name of this pass
    fn name() -> &'static str
    where
        Self: Sized;

    /// Executes the pass
    fn execute(
        &mut self,
        encoder: &mut wutengine_graphics::wgpu::CommandEncoder,
        viewport: &ViewportInfo,
    );
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

#[profiling::function]
pub(crate) fn render_frame() {
    log::debug!("Rendering frame");

    wutengine_graphics::surface::get_all_surface_textures();

    let world = WORLD_MANAGER.shared();

    let mut passes = RENDER_PIPELINE_MANAGER.pipeline.lock().unwrap();
    let mut cam_query = world.query::<(&Camera, Option<&Name>)>();

    let mut command_buffers: Vec<_> = Vec::new();

    for (_, (camera, name)) in cam_query.iter() {
        //TODO: Multithread?

        let cam_name = match name {
            Some(name_component) => name_component.0.as_str(),
            None => "(unnamed)",
        };

        command_buffers.push(render_camera(camera, cam_name, &mut passes));
    }

    wutengine_graphics::submit_command_buffers(command_buffers);

    wutengine_graphics::surface::present_all();
}

fn render_camera(
    camera: &Camera,
    name: &str,
    passes: &mut [RenderPassInfo],
) -> wgpu::CommandBuffer {
    profiling::function_scope!(name);

    log::debug!("Rendering camera: {name}");

    let mut encoder = wutengine_graphics::create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(format!("Camera {} Command Encoder", name).as_str()),
    });

    for pass in passes.iter_mut() {
        profiling::scope!(pass.name);

        log::debug!("Rendering pass: {}", pass.name);

        // pass.pass.execute(&mut encoder, &camera.get_viewport_info());
    }

    encoder.finish()
}
