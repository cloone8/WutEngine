//! Material asset

use core::any::Any;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::AssetHandle;
use crate::SerializedAsset;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "P: Serialize"))]
#[serde(bound(deserialize = "P: Deserialize<'de>"))]
pub struct SerializedMaterial<S, P> {
    pub shader: AssetHandle<S>,
    pub keywords: HashMap<String, u64>,
    pub parameters: HashMap<String, P>,
}

impl<S: Any, P: Any + Serialize + DeserializeOwned> SerializedAsset for SerializedMaterial<S, P> {}
