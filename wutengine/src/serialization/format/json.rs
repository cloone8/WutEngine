use std::error::Error;

use serde::{de::DeserializeOwned, Serialize};

use super::SerializationFormat;

#[derive(Debug)]
pub struct Json;

impl SerializationFormat for Json {
    type RawType = serde_json::Value;

    fn deserialize<T: DeserializeOwned>(data: Self::RawType) -> Result<T, Box<dyn Error>> {
        match serde_json::from_value::<T>(data) {
            Ok(obj) => Ok(obj),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn serialize<T: Serialize>(object: &T) -> Result<Self::RawType, Box<dyn Error>> {
        match serde_json::to_value(object) {
            Ok(val) => Ok(val),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn deserialize_from_reader<T: DeserializeOwned>(
        reader: impl std::io::Read,
    ) -> Result<T, Box<dyn Error>> {
        match serde_json::from_reader(reader) {
            Ok(obj) => Ok(obj),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn serialize_to_writer<T: Serialize>(
        object: &T,
        writer: impl std::io::Write,
    ) -> Result<(), Box<dyn Error>> {
        match serde_json::to_writer_pretty(writer, object) {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
}
