use wutengine_core::{
    id::{instance::InstanceID, KeyType},
    lookuptable::LookupTable,
    script::{abstractscript::AbstractScript, Script},
};

use crate::serialization::format::SerializationFormat;

#[derive(Debug)]
pub struct ScriptLoader<F: SerializationFormat> {
    script_id: KeyType,
    loader_fn: fn(F::RawType) -> Box<dyn AbstractScript>,
}

impl<F: SerializationFormat> ScriptLoader<F> {
    pub fn load(&self, data: F::RawType) -> Box<dyn AbstractScript> {
        (self.loader_fn)(data)
    }
}

impl<F: SerializationFormat> ScriptLoader<F> {
    pub fn new<T: Script>() -> Self {
        Self {
            script_id: T::script_id(),
            loader_fn: load_object::<T, F>,
        }
    }
}

impl<F: SerializationFormat> InstanceID for ScriptLoader<F> {
    fn id(&self) -> KeyType {
        self.script_id
    }
}

fn load_object<T: Script, F: SerializationFormat>(data: F::RawType) -> Box<dyn AbstractScript> {
    let deserialized = F::deserialize::<T>(data).unwrap();

    Box::new(deserialized)
}

pub struct ScriptLoaders<F: SerializationFormat> {
    pub(crate) loaders: LookupTable<ScriptLoader<F>>,
}

impl<F: SerializationFormat> ScriptLoaders<F> {
    pub fn new() -> Self {
        Self {
            loaders: LookupTable::new(),
        }
    }

    pub fn register_script<T: Script>(&mut self) -> &mut Self {
        self.loaders.insert(ScriptLoader::<F>::new::<T>());
        self
    }
}
