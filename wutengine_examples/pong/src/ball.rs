use serde::{Deserialize, Serialize};
use wutengine_core::script::{abstractscript::AbstractScript, Script};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ball {}

impl AbstractScript for Ball {}

impl Script for Ball {
    fn script_id() -> wutengine_core::id::KeyType {
        3294876123987
    }
}
