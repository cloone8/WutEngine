use std::fmt::Display;

use crate::WutEngine;

use super::{Renderable, WutEngineRenderer};

#[derive(Debug, Default)]
pub struct HeadlessRenderer;

impl WutEngineRenderer for HeadlessRenderer {
    const NAME: &'static str = "Headless";

    fn init(&mut self) {
        log::info!("Initialized headless backend")
    }

    fn render(&mut self, objects: &[Renderable]) {
        log::info!("Rendering engine state");
    }
}
