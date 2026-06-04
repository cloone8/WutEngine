//! Material asset

use core::any::Any;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::AssetHandle;
use crate::SerializedAsset;

/// The data for a single material
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "P: Serialize"))]
#[serde(bound(deserialize = "P: Deserialize<'de>"))]
pub struct SerializedMaterial<S, P> {
    /// The shader used by this material
    pub shader: AssetHandle<S>,

    /// The set keyword values for this material
    pub keywords: HashMap<String, u64>,

    /// The parameter values for this material
    pub parameters: HashMap<String, P>,
}

impl<S: Any, P: Any + Serialize + DeserializeOwned> SerializedAsset for SerializedMaterial<S, P> {}
