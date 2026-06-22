#![doc = include_str!("../../README.md")]

use wutengine::runtime::InitRuntimeConfig;
use wutengine::runtime::SystemManifest;
use wutengine::window::Window;
use wutengine::window::WindowConfig;

fn add_systems() -> SystemManifest {
    let mut manifest = SystemManifest::default();

    manifest
}

fn main() {
    println!("Hello, world!");

    wutengine::runtime::run(
        add_systems(),
        InitRuntimeConfig::default(),
        Some(Box::new(post_start)),
    )
    .expect("Failure while executing WutEngine runtime");
}

fn post_start() {
    log::info!("Starting WutEngine Editor");

    let main_editor_window = Window::create(WindowConfig {
        title: Some("WutEngine Editor".to_string()),
        resizable: true,
        size: (1920, 1080),
        fullscreen: None,
        ..Default::default()
    });
}
