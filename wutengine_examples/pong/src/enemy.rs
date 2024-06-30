use serde::{Deserialize, Serialize};
use wutengine_core::script::{abstractscript::AbstractScript, Script};

#[derive(Debug, Serialize, Deserialize)]
pub struct Enemy {}

impl AbstractScript for Enemy {}

impl Script for Enemy {
    fn script_id() -> wutengine_core::id::KeyType {
        810010
    }
}
