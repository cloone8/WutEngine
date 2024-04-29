use std::fmt::Display;

use crate::WutEngine;

use super::{Renderable, WutEngineRenderer};

pub struct HeadlessRenderer;

impl Default for HeadlessRenderer {
    fn default() -> Self {
        HeadlessRenderer {}
    }
}

impl WutEngineRenderer for HeadlessRenderer {
    const NAME: &'static str = "Headless";

    fn init(&mut self) {
        log::info!("Initialized headless backend")
    }

    fn render(&mut self, objects: &[Renderable]) {
        log::info!("Rendering engine state");
    }
}
