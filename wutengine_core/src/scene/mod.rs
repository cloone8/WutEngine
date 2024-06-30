use crate::{lookuptable::LookupTable, object::Object, script::ScriptData};

#[derive(Debug)]
pub struct Scene {
    pub objects: LookupTable<Object>,
    pub scripts: LookupTable<ScriptData>,
}
