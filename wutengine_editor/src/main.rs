use wutengine::builtins::components::camera::Camera;
use wutengine::builtins::components::material::Material;
use wutengine::builtins::components::mesh::Mesh;
use wutengine::command::Command;
use wutengine::core::{System, SystemPhase};
use wutengine::graphics::color::Color;
use wutengine::graphics::material::{MaterialData, MaterialParameter};
use wutengine::graphics::mesh::MeshData;
use wutengine::graphics::shader::ShaderSetId;
use wutengine::math::Vec3;
use wutengine::runtime::RuntimeInitializer;
use wutengine::world::World;
use wutengine::SystemFunction;
use wutengine::{
    command::{FullscreenType, OpenWindowParams},
    graphics::windowing::WindowIdentifier,
    plugin::EnginePlugin,
    renderer::OpenGLRenderer,
    EngineCommand, EngineEvent,
};

fn main() {
    let mut runtime = RuntimeInitializer::new();

    runtime.add_plugin::<WutEngineEditorPlugin>();
    runtime.run::<OpenGLRenderer>().unwrap();
}

struct WutEngineEditorPlugin;

impl EnginePlugin for WutEngineEditorPlugin {
    fn build() -> Self
    where
        Self: Sized,
    {
        WutEngineEditorPlugin
    }

    fn on_event(&mut self, event: &EngineEvent) -> Vec<EngineCommand> {
        match event {
            EngineEvent::RuntimeStart => editor_start(),
        }
    }
}

fn editor_start() -> Vec<EngineCommand> {
    vec![
        EngineCommand::OpenWindow(OpenWindowParams {
            id: WindowIdentifier::new("main"),
            title: "WutEngine Editor".to_owned(),
            mode: FullscreenType::Windowed,
            ignore_existing: false,
        }),
        EngineCommand::AddSystem(System {
            phase: SystemPhase::RuntimeStart,
            func: SystemFunction::Immutable(init_system),
        }),
    ]
}

fn init_system(commands: &mut Command, _world: &World) {
    let camera = Camera {
        display: WindowIdentifier::new("main"),
        clear_color: Color::rgb(0.0, 0.0, 0.0),
    };

    let mut basic_triangle_mesh = MeshData::new();

    basic_triangle_mesh.positions = vec![
        Vec3::new(-0.5, -0.5, 0.0),
        Vec3::new(0.5, -0.5, 0.0),
        Vec3::new(0.0, 0.5, 0.0),
    ];

    let basic_triangle = Mesh::new(basic_triangle_mesh);

    let matdata = MaterialData {
        shader: ShaderSetId::new("unlit"),
        parameters: wutengine::macros::map!(
            "baseColor".to_owned() => MaterialParameter::Color(Color::rgb(1.0, 215.0 / 255.0, 0.0))
        ),
    };

    commands
        .entity()
        .spawn_with_components(vec![Box::new(camera)]);

    commands.entity().spawn_with_components(vec![
        Box::new(basic_triangle),
        Box::new(Material::new(matdata.clone())),
    ]);
}
