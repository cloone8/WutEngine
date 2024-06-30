use serde::{Deserialize, Serialize};
use wutengine::script::{abstractscript::AbstractScript, Script};

#[derive(Debug, Serialize, Deserialize)]
pub struct Enemy {}

impl AbstractScript for Enemy {}

impl Script for Enemy {
    fn script_id() -> wutengine::id::KeyType {
        810010
    }
}
