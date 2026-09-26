//! Shader pipeline caching

use alloc::sync::Arc;
use std::sync::LazyLock;

use dashmap::DashMap;
use wutengine_assets::assets::shader::PrecompiledShader;
use wutengine_assets::assets::shader::ShaderHash;

use crate::shader::CompiledShader;

use super::GraphicsCache;

static SHADER_COMPILATION_CACHE: LazyLock<GraphicsCache<ShaderHash, CompiledShader>> =
    LazyLock::new(Default::default);

/// Precompiled variants that haven't been turned into a [`CompiledShader`] yet
static PRECOMPILED: LazyLock<DashMap<ShaderHash, PrecompiledShader>> =
    LazyLock::new(Default::default);

/// Tries to find a given shader variant in the global cache
#[inline]
pub(crate) fn find(key: &ShaderHash) -> Option<Arc<CompiledShader>> {
    SHADER_COMPILATION_CACHE.find(key)
}

/// Inserts the given compiled shader variant under the given key. If the variant already exists,
/// does not insert the new variant and simply returns the already existing one
#[inline]
pub(crate) fn insert(key: ShaderHash, variant: CompiledShader) -> Arc<CompiledShader> {
    SHADER_COMPILATION_CACHE.insert(key, variant)
}

/// Stores a precompiled variant until it is needed
#[inline]
pub(crate) fn insert_precompiled(shader: PrecompiledShader) {
    PRECOMPILED.insert(shader.hash, shader);
}

/// Removes and returns the precompiled variant with the given hash, if registered
#[inline]
pub(crate) fn take_precompiled(key: &ShaderHash) -> Option<PrecompiledShader> {
    PRECOMPILED.remove(key).map(|(_, shader)| shader)
}
