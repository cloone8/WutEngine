use core::fmt::Debug;
use std::{
    error::Error,
    io::{Read, Write},
};

use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "serialization_json")]
pub mod json;

pub trait SerializationFormat {
    type RawType: Serialize + DeserializeOwned + Debug;

    fn deserialize<T: DeserializeOwned>(data: Self::RawType) -> Result<T, Box<dyn Error>>;

    fn serialize<T: Serialize>(object: &T) -> Result<Self::RawType, Box<dyn Error>>;

    fn deserialize_from_reader<T: DeserializeOwned>(reader: impl Read)
        -> Result<T, Box<dyn Error>>;

    fn serialize_to_writer<T: Serialize>(
        object: &T,
        writer: impl Write,
    ) -> Result<(), Box<dyn Error>>;
}
