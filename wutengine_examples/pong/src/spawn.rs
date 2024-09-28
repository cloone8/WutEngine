//! Contains the spawning plugin for the game

use std::time::Instant;

use wutengine::builtins::components::util::FramerateCounter;
use wutengine::builtins::components::CameraType::{self};
use wutengine::builtins::components::{Camera, Material, Mesh, Name, Transform};
use wutengine::command::{Command, FullscreenType, OpenWindowParams};
use wutengine::graphics::color::Color;
use wutengine::graphics::material::{MaterialData, MaterialParameter};
use wutengine::graphics::mesh::{IndexBuffer, MeshData};
use wutengine::graphics::shader::ShaderSetId;
use wutengine::graphics::windowing::WindowIdentifier;
use wutengine::map;
use wutengine::math::{vec3, Quat, Vec3};
use wutengine::plugins::WutEnginePlugin;

use crate::BallData;

/// Plugin that only injects the initial components to get the game started
pub(crate) struct PongStarterPlugin;

impl WutEnginePlugin for PongStarterPlugin {
    fn on_start(&mut self, commands: &mut Command) {
        let mut rectangle_mesh_data = MeshData::new();
        rectangle_mesh_data.positions = vec![
            Vec3::new(0.5, 0.5, 0.0),
            Vec3::new(0.5, -0.5, 0.0),
            Vec3::new(-0.5, -0.5, 0.0),
            Vec3::new(-0.5, 0.5, 0.0),
        ];

        rectangle_mesh_data.indices = IndexBuffer::U32(vec![0, 1, 3, 1, 2, 3]);
        let rectangle_mesh = Mesh::new(rectangle_mesh_data);

        make_window(commands);
        make_camera(commands);
        make_player(commands, rectangle_mesh.clone());
        make_enemy(commands, rectangle_mesh.clone());
        make_ball(commands, rectangle_mesh.clone());
    }
}

/// Opens the window
fn make_window(commands: &mut Command) {
    commands.window().open(OpenWindowParams {
        id: WindowIdentifier::new("main"),
        title: "Pong - WutEngine".to_string(),
        mode: FullscreenType::Windowed,
        ignore_existing: false,
    });
}

/// Creates the player entity
fn make_player(commands: &mut Command, mesh: Mesh) {
    commands
        .entity()
        .spawn()
        .with_component(Name::new("Player"))
        .with_component(Transform::with_pos_rot_scale(
            vec3(-1.1, 0.0, 0.0),
            Quat::IDENTITY,
            vec3(0.125, 0.4, 1.0),
        ))
        .with_component(mesh)
        .with_component(Material::new(MaterialData {
            shader: ShaderSetId::new("unlit"),
            parameters: map![
                "baseColor" => MaterialParameter::Color(Color::BLUE)
            ],
        }))
        .build();
}

/// Creates the enemy entity
fn make_enemy(commands: &mut Command, mesh: Mesh) {
    commands
        .entity()
        .spawn()
        .with_component(Name::new("Enemy"))
        .with_component(Transform::with_pos_rot_scale(
            vec3(1.1, 0.0, 0.0),
            Quat::IDENTITY,
            vec3(0.125, 0.4, 1.0),
        ))
        .with_component(mesh)
        .with_component(Material::new(MaterialData {
            shader: ShaderSetId::new("unlit"),
            parameters: map![
                "baseColor" => MaterialParameter::Color(Color::RED)
            ],
        }))
        .build();
}

/// Creates the ball entity
fn make_ball(commands: &mut Command, mesh: Mesh) {
    commands
        .entity()
        .spawn()
        .with_component(Name::new("Ball"))
        .with_component(BallData {
            start: Instant::now(),
        })
        .with_component(Transform::with_pos_rot_scale(
            Vec3::ZERO,
            Quat::IDENTITY,
            vec3(0.07, 0.07, 0.07),
        ))
        .with_component(mesh)
        .with_component(Material::new(MaterialData {
            shader: ShaderSetId::new("unlit"),
            parameters: map![
                "baseColor" => MaterialParameter::Color(Color::WHITE)
            ],
        }))
        .build();
}

/// Creates the camera entity
fn make_camera(commands: &mut Command) {
    commands
        .entity()
        .spawn()
        .with_component(Name::new("Camera"))
        .with_component(FramerateCounter::new())
        .with_component(Transform::with_pos(Vec3::new(0.0, 0.0, -3.0)))
        .with_component(Camera {
            display: WindowIdentifier::new("main"),
            clear_color: Color::BLACK,
            camera_type: CameraType::Orthographic(2.0),
        })
        .build();
}
