//! Entity asset

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedEntity {
    pub name: String,
}
