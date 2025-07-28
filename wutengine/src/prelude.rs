//! Common types, traits and functions

pub use crate::asset::{Asset, AssetHandle};
pub use crate::builtin::*;
pub use crate::component::{Component, ComponentContext, ComponentId};
pub use crate::config::StaticRuntimeConfig;
pub use crate::gameobject::{GameObject, GameObjectId};
pub use crate::graphics::{material::Material, mesh::Mesh, texture::Texture};
pub use crate::math::{
    Mat3, Mat3A, Mat4, Vec2, Vec3, Vec3A, Vec4, mat3, mat3a, mat4, vec2, vec3, vec3a, vec4,
};
