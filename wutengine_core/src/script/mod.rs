use abstractscript::AbstractScript;
use serde::{de::DeserializeOwned, Serialize};

use crate::id::{instance::InstanceID, KeyType};

pub mod abstractscript;
pub mod frame;
pub mod start;

#[derive(Debug)]
pub struct ScriptData {
    pub instance_id: KeyType,
    pub script_id: KeyType,
    pub script: Box<dyn AbstractScript>,
}

impl InstanceID for ScriptData {
    fn id(&self) -> KeyType {
        self.instance_id
    }
}

pub trait Script: AbstractScript + Sized + Serialize + DeserializeOwned + 'static {
    fn script_id() -> KeyType;
}
