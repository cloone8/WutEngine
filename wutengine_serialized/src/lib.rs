mod ordered_vec;
pub use ordered_vec::*;

use relative_path::RelativePathBuf;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Scene {
    id: Uuid,
    name: String
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    name: String,
    config: ProjectConfig,
    scenes: Vec<RelativePathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectConfig {
 
}
