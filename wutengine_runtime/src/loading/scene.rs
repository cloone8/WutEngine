use std::{fs::File, io::BufReader, path::Path};

use wutengine_core::{lookuptable::LookupTable, object::Object, scene::Scene, script::ScriptData};

use crate::serialization::{format::SerializationFormat, scene::SerializedScene};

use super::script::ScriptLoader;

#[derive(Debug)]
pub struct SceneLoader;

impl SceneLoader {
    pub fn load<F: SerializationFormat>(
        source: &Path,
        script_loaders: &LookupTable<ScriptLoader<F>>,
    ) -> Result<Scene, ()> {
        let source_file = File::open(source).unwrap();
        let reader = BufReader::new(source_file);
        let serialized_scene: SerializedScene<F> = F::deserialize_from_reader(reader).unwrap();

        let mut scene = Scene {
            objects: LookupTable::new(),
            scripts: LookupTable::new(),
        };

        for obj in serialized_scene.objects {
            let new_object = Object::new(obj.instance_id, obj.name, obj.transform.into());

            scene.objects.insert(new_object);

            for script in obj.scripts {
                let loaded = script_loaders
                    .get(script.script_id)
                    .unwrap()
                    .load(script.data);

                let script_data = ScriptData {
                    instance_id: script.instance_id,
                    script_id: script.script_id,
                    script: loaded,
                };

                scene.scripts.insert(script_data);
            }
        }

        Ok(scene)
    }
}
