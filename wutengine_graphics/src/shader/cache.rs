use core::borrow::Borrow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use wutengine_util::hash::nohash_hasher::IntMap;

#[derive(Debug)]
pub(crate) struct ShaderCache {
    /// A nested map, where the outer map maps from shader name to shader variants,
    /// and the inner map maps from shader variant to compiled module
    compiled: RwLock<HashMap<String, IntMap<u64, Arc<wgpu::ShaderModule>>>>,
}

impl ShaderCache {
    pub(crate) fn new() -> Self {
        Self {
            compiled: RwLock::new(HashMap::default()),
        }
    }

    pub(crate) fn find<Q>(
        &self,
        shader_name: &Q,
        keyword_hash: u64,
    ) -> Option<Arc<wgpu::ShaderModule>>
    where
        String: Borrow<Q>,
        Q: core::hash::Hash + Eq,
    {
        let compiled = self.compiled.read().unwrap();

        compiled
            .get(shader_name)
            .and_then(|variants| variants.get(&keyword_hash))
            .map(Arc::clone)
    }

    pub(crate) fn insert(
        &self,
        shader_name: String,
        keyword_hash: u64,
        module: wgpu::ShaderModule,
    ) -> Arc<wgpu::ShaderModule> {
        let as_arc = Arc::new(module);
        let mut compiled = self.compiled.write().unwrap();

        _ = compiled
            .entry(shader_name)
            .or_default()
            .insert(keyword_hash, Arc::clone(&as_arc));

        as_arc
    }
}
