use crate::{fastmap::FastMap, object::Object, script::ScriptData};

#[derive(Debug)]
pub struct Scene {
    pub objects: FastMap<Object>,
    pub scripts: FastMap<ScriptData>,
}
