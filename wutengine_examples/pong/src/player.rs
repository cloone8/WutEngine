use serde::{Deserialize, Serialize};
use wutengine_core::script::{abstractscript::AbstractScript, start::Start, Script};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {}

impl AbstractScript for Player {
    fn as_start(&mut self) -> Option<&mut dyn Start> {
        Some(self)
    }
}

impl Script for Player {
    fn script_id() -> wutengine_core::id::KeyType {
        1
    }
}

impl Start for Player {
    fn on_start(&mut self) {
        //TODO: Test window-opening code
        println!("Hello!");
    }
}
