//! Material functionality

use core::num::{NonZero, NonZeroUsize};
use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::sync::Arc;

use nohash_hasher::IntMap;
use wgpu::BufferUsages;
use wutengine_util_macro::{VariantName, unique_id_type64};

use super::GFX_DEVICE;
use super::sampler::Sampler;
use super::shader::{self, CompileErr, CompiledShader, CompiledShaderId, ParameterBinding, Shader};
use super::texture::Texture;

pub(crate) mod bind_group;
mod parameter;

pub use parameter::*;

unique_id_type64! {
    /// The unique identifier for a [Material]
    pub(crate) MaterialId
}

/// A material used for rendering
#[derive(Debug)]
pub(crate) struct Material {
    id: MaterialId,

    pub(crate) compiled_shader: Arc<CompiledShader>,

    bind_groups: Vec<MaterialBindGroup>,
    // /// The shader this material uses. Used for compilation.
    // shader: Arc<Shader>,

    // /// The concrete keyword values in this material. Used for compilation
    // keywords: HashMap<String, u64>,
}

#[derive(Debug)]
pub(crate) struct MaterialBindGroup {
    handle: wgpu::BindGroup,
    buffer: Option<wgpu::Buffer>,
    entries: IntMap<u32, MaterialBindGroupEntry>,
}

#[derive(Debug, derive_more::IsVariant)]
pub(crate) enum MaterialBindGroupEntry {
    Buffer { offset: usize, size: NonZeroUsize },
    Texture(Texture),
    Sampler(Sampler),
}

impl MaterialBindGroupEntry {
    fn to_wgpu_binding_resource<'a>(
        &'a self,
        group_buffer: &'a wgpu::Buffer,
    ) -> wgpu::BindingResource<'a> {
        match self {
            Self::Texture(texture) => wgpu::BindingResource::TextureView(&texture.get_view()),
            Self::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler.get_wgpu()),
            Self::Buffer { offset, size } => wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: group_buffer,
                offset: u64::try_from(*offset).unwrap(),
                size: Some(NonZero::<u64>::new(u64::try_from(size.get()).unwrap()).unwrap()),
            }),
        }
    }
}

/// A possible parameter value for a material
#[derive(
    Debug, Clone, derive_more::IsVariant, derive_more::Unwrap, derive_more::TryUnwrap, VariantName,
)]
pub enum ParameterValue {
    // /// A texture parameter
    // Texture2D(Texture),
    /// A sampler parameter
    Sampler(Sampler),
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub(crate) enum MaterialErr {
    Keywords(Box<VerifyErr>),
    Compile(Box<CompileErr>),
}

/// Public API
impl Material {
    /// Creates a new native material from the given shader, with no keywords set
    pub(crate) fn new(
        shader: Arc<Shader>,
        keywords: HashMap<String, u64>,
        parameters: &[IntMap<u32, ParameterValue>],
    ) -> Result<Self, MaterialErr> {
        profiling::function_scope!();

        Self::verify_keywords(&keywords, &shader)?;

        let compiled = shader::compile(&shader, &keywords)?;

        let bind_groups = Self::create_bind_groups(&compiled, parameters);

        Ok(Self {
            id: MaterialId::new(),
            compiled_shader: compiled,
            bind_groups,
        })
    }

    /// Returns the ID of this material
    #[inline(always)]
    pub(crate) const fn id(&self) -> MaterialId {
        self.id
    }

    /// Returns the ID of the compiled shader of this material
    #[inline(always)]
    pub(crate) fn compiled_shader_id(&self) -> CompiledShaderId {
        self.compiled_shader.id
    }

    pub(crate) fn set_parameter(&mut self, binding: ParameterBinding, value: ParameterValue) {
        let group = &mut self.bind_groups[binding.group as usize];
        //TODO: Check value type, and update group. If texture/sampler, remake bind group. If buffer, just update buffer
        sdfasdfha
    }
}

/// Private API
impl Material {
    fn create_bind_groups<'a>(
        shader: &CompiledShader,
        parameters: &[IntMap<u32, ParameterValue>],
    ) -> Vec<MaterialBindGroup> {
        profiling::function_scope!();

        let mut bind_groups = Vec::new();

        for (group, parameters) in parameters.iter().enumerate() {
            let group_buf = GFX_DEVICE.create_buffer(&wgpu::wgt::BufferDescriptor {
                label: Some(format!("Shader {} bind group {} buffer", shader.name, group).as_str()),
                size: 1u64.next_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT),
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                mapped_at_creation: false,
            });

            let mut entries = HashMap::default();

            for (binding, parameter) in parameters {
                let binding = *binding;

                match parameter {
                    ParameterValue::Sampler(sampler) => {
                        entries.insert(binding, MaterialBindGroupEntry::Sampler(sampler.clone()));
                    }
                }
            }

            let mut entries_flat: Vec<wgpu::BindGroupEntry> = Vec::new();

            for (binding, entry) in entries.iter() {
                entries_flat.push(wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: entry.to_wgpu_binding_resource(&group_buf),
                });
            }

            let entries_flat: Vec<_> = entries
                .iter()
                .map(|(k, val)| wgpu::BindGroupEntry {
                    binding: *k,
                    resource: val.to_wgpu_binding_resource(&group_buf),
                })
                .collect();

            let native_bind_group = GFX_DEVICE.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(format!("Shader {} bind group {}", shader.name, group).as_str()),
                layout: &shader.user_param_group_layout[group],
                entries: &entries_flat,
            });

            bind_groups.push(MaterialBindGroup {
                handle: native_bind_group,
                buffer: None,
                entries,
            });
        }

        bind_groups
    }

    /// Verifies that all keyword/value pairs in `to_verify` would be valid to set when using the given shader.
    fn verify_keywords(
        to_verify: &HashMap<String, u64>,
        shader: &Shader,
    ) -> Result<(), Box<VerifyErr>> {
        for (keyword, value) in to_verify.iter() {
            Self::verify_single_keyword(keyword.as_str(), *value, shader)?;
        }

        for keyword in shader.allowed_keywords.keys() {
            if !to_verify.contains_key(keyword) {
                return Err(Box::new(VerifyErr::MissingKeyword(keyword.clone())));
            }
        }

        Ok(())
    }

    /// Verifies that a given keyword/value combination would be valid to set when using the given shader.
    fn verify_single_keyword(key: &str, value: u64, shader: &Shader) -> Result<(), Box<VerifyErr>> {
        if let Some(allowed_keyword_values) = shader.allowed_keywords.get(key) {
            if !allowed_keyword_values.contains(&value) {
                return Err(Box::new(VerifyErr::InvalidKeywordValue(
                    key.to_owned(),
                    value,
                    allowed_keyword_values.clone(),
                )));
            }
        } else {
            return Err(Box::new(VerifyErr::UnknownKeyword(key.to_owned())));
        }

        Ok(())
    }
}

/// Keywords on a material are not compatible with its shader
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum VerifyErr {
    /// A keyword is set on the material that is not present
    #[display("Keyword on material not known by the used shader: {}", _0)]
    UnknownKeyword(#[error(not(source))] String),

    #[display("Keyword is missing: {}", _0)]
    MissingKeyword(#[error(not(source))] String),

    /// A keyword has a value that is not valid for the shader
    #[display("Invalid value for keyword {} with allowed range {}..={}: {}", _0, _2.start(), _2.end(), _1)]
    InvalidKeywordValue(String, u64, RangeInclusive<u64>),
}
