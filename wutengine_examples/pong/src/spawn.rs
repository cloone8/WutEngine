//! Contains the spawning plugin for the game

use std::time::Instant;

use wutengine::builtins::assets::{Material, Mesh};
use wutengine::builtins::components::util::FramerateCounter;
use wutengine::builtins::components::CameraType::{self};
use wutengine::builtins::components::{Camera, InputHandler, StaticMeshRenderer, Transform};
use wutengine::gameobject::GameObject;
use wutengine::graphics::color::Color;
use wutengine::graphics::material::{MaterialData, MaterialParameter};
use wutengine::graphics::mesh::{IndexBuffer, MeshData};
use wutengine::graphics::shader::ShaderSetId;
use wutengine::graphics::windowing::WindowIdentifier;
use wutengine::math::{vec3, Quat, Vec3};
use wutengine::plugins::WutEnginePlugin;
use wutengine::windowing::{FullscreenType, OpenWindowParams};
use wutengine::{map, plugins};

use crate::{BallData, PlayerMovement};

/// Plugin that only injects the initial components to get the game started
pub(crate) struct PongStarterPlugin;

impl WutEnginePlugin for PongStarterPlugin {
    fn on_start(&mut self, context: &mut plugins::Context) {
        let mut rectangle_mesh_data = MeshData::new();
        rectangle_mesh_data.positions = vec![
            Vec3::new(0.5, 0.5, 0.0),
            Vec3::new(0.5, -0.5, 0.0),
            Vec3::new(-0.5, -0.5, 0.0),
            Vec3::new(-0.5, 0.5, 0.0),
        ];

        rectangle_mesh_data.indices = IndexBuffer::U32(vec![0, 1, 3, 1, 2, 3]);
        let rectangle_mesh = Mesh::new(rectangle_mesh_data);

        make_window(context);
        make_camera(context);
        make_player(context, rectangle_mesh.clone());
        make_enemy(context, rectangle_mesh.clone());
        make_ball(context, rectangle_mesh.clone());
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Opens the window
fn make_window(context: &mut plugins::Context) {
    context.windows.open(OpenWindowParams {
        id: WindowIdentifier::new("main"),
        title: "Pong - WutEngine".to_string(),
        mode: FullscreenType::Windowed,
        ignore_existing: false,
    });
}

/// Creates the player entity
fn make_player(context: &mut plugins::Context, mesh: Mesh) {
    let mut player = GameObject::new(Some("Player".to_string()));

    player.add_component(Box::new(InputHandler::new()));
    player.add_component(Box::new(PlayerMovement::new()));
    player.add_component(Box::new(Transform::with_pos_rot_scale(
        vec3(-1.1, 0.0, 0.0),
        Quat::IDENTITY,
        vec3(0.125, 0.4, 1.0),
    )));
    player.add_component(Box::new(StaticMeshRenderer {
        mesh,
        material: Material::new(MaterialData {
            shader: ShaderSetId::new("unlit"),
            parameters: map![
                "baseColor" => MaterialParameter::Color(Color::BLUE)
            ],
        }),
    }));

    context.engine.spawn_gameobject(player);
}

/// Creates the enemy entity
fn make_enemy(context: &mut plugins::Context, mesh: Mesh) {
    let mut enemy = GameObject::new(Some("Enemy".to_string()));

    enemy.add_component(Box::new(Transform::with_pos_rot_scale(
        vec3(1.1, 0.0, 0.0),
        Quat::IDENTITY,
        vec3(0.125, 0.4, 1.0),
    )));
    enemy.add_component(Box::new(StaticMeshRenderer {
        mesh,
        material: Material::new(MaterialData {
            shader: ShaderSetId::new("unlit"),
            parameters: map![
                "baseColor" => MaterialParameter::Color(Color::RED)
            ],
        }),
    }));

    context.engine.spawn_gameobject(enemy);
}

/// Creates the ball entity
fn make_ball(context: &mut plugins::Context, mesh: Mesh) {
    let mut ball = GameObject::new(Some("Ball".to_string()));

    ball.add_component(Box::new(BallData {
        start: Instant::now(),
    }));

    ball.add_component(Box::new(Transform::with_pos_rot_scale(
        Vec3::ZERO,
        Quat::IDENTITY,
        vec3(0.07, 0.07, 0.07),
    )));
    ball.add_component(Box::new(StaticMeshRenderer {
        mesh,
        material: Material::new(MaterialData {
            shader: ShaderSetId::new("unlit"),
            parameters: map![
                "baseColor" => MaterialParameter::Color(Color::WHITE)
            ],
        }),
    }));

    context.engine.spawn_gameobject(ball);
}

/// Creates the camera entity
fn make_camera(context: &mut plugins::Context) {
    let mut camera = GameObject::new(Some("Camera".to_string()));

    camera.add_component(Box::new(FramerateCounter::new()));

    camera.add_component(Box::new(Transform::with_pos(Vec3::new(0.0, 0.0, -3.0))));
    camera.add_component(Box::new(Camera {
        display: WindowIdentifier::new("main"),
        clear_color: Color::BLACK,
        camera_type: CameraType::Orthographic(2.0),
    }));
    context.engine.spawn_gameobject(camera);
}
