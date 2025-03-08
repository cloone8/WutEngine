//! Contains the spawning plugin for the game

use wutengine::builtins::assets::{Material, Mesh};
use wutengine::builtins::components::camera::CameraType::{self};
use wutengine::builtins::components::physics::RectangleCollider2D;
use wutengine::builtins::components::util::FramerateCounter;
use wutengine::builtins::components::{
    camera::Camera, input::InputHandler, static_mesh_renderer::StaticMeshRenderer,
    transform::Transform,
};
use wutengine::gameobject::GameObject;
use wutengine::graphics::color::Color;
use wutengine::graphics::material::{MaterialData, MaterialParameter};
use wutengine::graphics::mesh::{IndexBuffer, IndexType, MeshData};
use wutengine::graphics::shader::ShaderId;
use wutengine::math::{Quat, Vec2, Vec3, random, vec2, vec3};
use wutengine::plugins::WutEnginePlugin;
use wutengine::windowing::WindowIdentifier;
use wutengine::windowing::{self, FullscreenType, OpenWindowParams};
use wutengine::{map, plugins};

use crate::balldata::BallData;
use crate::enemy::Enemy;
use crate::player::PlayerMovement;

/// Plugin that only injects the initial components to get the game started
#[derive(Debug)]
pub(crate) struct PongStarterPlugin;

impl WutEnginePlugin for PongStarterPlugin {
    fn on_start(&mut self, context: &mut plugins::Context) {
        let mut rectangle_mesh = Mesh::new();

        rectangle_mesh.set_vertex_positions(vec![
            Vec3::new(0.5, 0.5, 0.0),
            Vec3::new(0.5, -0.5, 0.0),
            Vec3::new(-0.5, -0.5, 0.0),
            Vec3::new(-0.5, 0.5, 0.0),
        ]);

        rectangle_mesh.set_indices(vec![0u32, 1, 3, 1, 2, 3]);
        rectangle_mesh.set_index_type(IndexType::Triangles);

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
    let displays = windowing::display::available_displays();

    let _primary = displays.primary();

    context.windows.open(OpenWindowParams {
        id: WindowIdentifier::new("main"),
        title: "Pong - WutEngine".to_string(),
        // mode: FullscreenType::BorderlessFullscreen(primary.clone()),
        mode: FullscreenType::Windowed,
        ignore_existing: false,
    });
}

/// Creates the player entity
fn make_player(context: &mut plugins::Context, mesh: Mesh) {
    let mut player = GameObject::new(Some("Player".to_string()));

    player.add_component(InputHandler::new());
    player.add_component(PlayerMovement::new());
    player.add_component(RectangleCollider2D::new(Vec2::ZERO, Vec2::ONE));
    player.add_component(Transform::with_pos_rot_scale(
        vec3(-1.1, 0.0, 0.0),
        Quat::IDENTITY,
        vec3(0.125, 0.4, 1.0),
    ));

    let mut player_material = Material::new();
    player_material.set_shader(Some(ShaderId::new("unlit")));
    player_material.set_color("baseColor", Color::BLUE);

    player.add_component(StaticMeshRenderer {
        mesh,
        material: player_material,
    });

    context.engine.spawn_gameobject(player);
}

/// Creates the enemy entity
fn make_enemy(context: &mut plugins::Context, mesh: Mesh) {
    let mut enemy = GameObject::new(Some("Enemy".to_string()));

    enemy.add_component(Transform::with_pos_rot_scale(
        vec3(1.1, 0.0, 0.0),
        Quat::IDENTITY,
        vec3(0.125, 0.4, 1.0),
    ));
    enemy.add_component(RectangleCollider2D::new(Vec2::ZERO, Vec2::ONE));

    let mut enemy_material = Material::new();
    enemy_material.set_shader(Some(ShaderId::new("unlit")));
    enemy_material.set_color("baseColor", Color::RED);

    enemy.add_component(StaticMeshRenderer {
        mesh,
        material: enemy_material,
    });
    enemy.add_component(Enemy::new(0.5, 0.9));

    context.engine.spawn_gameobject(enemy);
}

/// Creates the ball entity
fn make_ball(context: &mut plugins::Context, mesh: Mesh) {
    let mut ball = GameObject::new(Some("Ball".to_string()));

    ball.add_component(BallData {
        speed: 0.8,
        direction: vec2(1.0, 0.0) * random::simple::sign(),
    });

    ball.add_component(Transform::with_pos_rot_scale(
        vec3(0.0, 0.0, -0.01),
        Quat::IDENTITY,
        vec3(0.07, 0.07, 0.07),
    ));
    ball.add_component(RectangleCollider2D::new(Vec2::ZERO, Vec2::ONE));

    let mut ball_material = Material::new();
    ball_material.set_shader(Some(ShaderId::new("unlit")));
    ball_material.set_color("baseColor", Color::WHITE);

    ball.add_component(StaticMeshRenderer {
        mesh,
        material: ball_material,
    });

    context.engine.spawn_gameobject(ball);
}

/// Creates the camera entity
fn make_camera(context: &mut plugins::Context) {
    let mut camera = GameObject::new(Some("Camera".to_string()));

    camera.add_component(FramerateCounter::new());

    camera.add_component(Transform::with_pos(Vec3::new(0.0, 0.0, -3.0)));
    camera.add_component(Camera {
        display: WindowIdentifier::new("main"),
        clear_color: Color::BLACK,
        camera_type: CameraType::Orthographic(2.0),
    });
    context.engine.spawn_gameobject(camera);
}
