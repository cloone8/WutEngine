use wutengine::builtins::components::camera::Camera;
use wutengine::builtins::components::material::Material;
use wutengine::builtins::components::mesh::Mesh;
use wutengine::graphics::material::{MaterialData, MaterialParameter};
use wutengine::graphics::mesh::MeshData;
use wutengine::graphics::shader::ShaderSetId;
use wutengine::log::{self, ComponentLogConfig, LogConfig};
use wutengine::math::Vec3;
use wutengine::runtime::RuntimeInitializer;
use wutengine::{
    command::{Command, FullscreenType, OpenWindowParams},
    core::{System, SystemPhase},
    graphics::{color::Color, windowing::WindowIdentifier},
    plugin::EnginePlugin,
    renderer::OpenGLRenderer,
    world::World,
    EngineCommand, SystemFunction,
};

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.add_plugin::<PongStarterPlugin>();
    runtime.with_log_config(LogConfig {
        runtime: Some(ComponentLogConfig {
            min_level: log::LevelFilter::Info,
            output: log::LogOutput::StdOut,
        }),
        ..Default::default()
    });
    runtime.run::<OpenGLRenderer>().unwrap();
}

struct PongStarterPlugin;

impl EnginePlugin for PongStarterPlugin {
    fn build() -> Self
    where
        Self: Sized,
    {
        PongStarterPlugin
    }

    fn on_event(&mut self, event: &wutengine::EngineEvent) -> Vec<EngineCommand> {
        let mut response = Vec::new();

        match event {
            wutengine::EngineEvent::RuntimeStart => {
                log::info!("Received runtime start event");

                response.push(EngineCommand::AddSystem(System {
                    phase: SystemPhase::RuntimeStart,
                    func: SystemFunction::Immutable(init_system),
                }));
            }
        }

        response
    }
}

fn init_system(commands: &mut Command, world: &World) {
    commands.window().open(OpenWindowParams {
        id: WindowIdentifier::new("pong"),
        title: "Pong - WutEngine Example".to_owned(),
        ignore_existing: false,
        mode: FullscreenType::Windowed,
    });

    let camera = Camera {
        display: WindowIdentifier::new("pong"),
        clear_color: Color::rgb(0.2, 0.3, 0.3),
    };

    let mut left_triangle_mesh = MeshData::new();

    left_triangle_mesh.positions = vec![
        Vec3::new(-1.0, -0.5, 0.0),
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(-0.5, 0.5, 0.0),
    ];

    let left_triangle = Mesh::new(left_triangle_mesh);

    let mut right_triangle_mesh = MeshData::new();

    right_triangle_mesh.positions = vec![
        Vec3::new(0.0, -0.5, 0.0),
        Vec3::new(1.0, -0.5, 0.0),
        Vec3::new(0.5, 0.5, 0.0),
    ];

    let right_triangle = Mesh::new(right_triangle_mesh);

    let matdata = MaterialData {
        shader: ShaderSetId::new("unlit"),
        parameters: wutengine::macros::map!(
            "baseColor".to_owned() => MaterialParameter::Color(Color::rgb(0.5, 0.0, 0.0))
        ),
    };

    commands.entity().spawn_with_components(vec![
        Box::new(camera),
        Box::new(Material::new(matdata.clone())),
    ]);

    commands.entity().spawn_with_components(vec![
        Box::new(left_triangle),
        Box::new(Material::new(matdata.clone())),
    ]);

    commands.entity().spawn_with_components(vec![
        Box::new(right_triangle),
        Box::new(Material::new(matdata.clone())),
    ]);
}
