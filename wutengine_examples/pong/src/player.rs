use serde::{Deserialize, Serialize};
use wutengine_core::script::{abstractscript::AbstractScript, Script};

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {}

impl AbstractScript for Player {}

impl Script for Player {
    fn script_id() -> wutengine_core::id::KeyType {
        1
    }
}
