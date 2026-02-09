//! Material functionality

use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::sync::Arc;

use wutengine_util_macro::{VariantName, unique_id_type64};

use crate::map;

use super::sampler::Sampler;
use super::shader::{self, CompiledShaderId, Shader};

unique_id_type64! {
    /// The unique identifier for a [NativeMaterial]
    pub(crate) MaterialId
}

/// A material used for rendering
#[derive(Debug)]
pub(crate) struct NativeMaterial {
    id: MaterialId,

    /// The cached compiled shader ID
    compiled_shader_id: CompiledShaderId,

    /// The shader this material uses
    shader: Arc<Shader>,

    /// The overridden keywords in this material.
    /// Any keywords present in [Self::shader] but not present in this map
    /// are set to `0`
    keywords: HashMap<String, u64>,

    keywords_with_defaults: HashMap<String, u64>,

    /// Any parameters set on this material. Must match
    /// the supported parameters of the compiled shader variant
    /// of this material
    parameters: HashMap<String, Option<ParameterValue>>,
}

/// A possible parameter value for a material
#[derive(
    Debug, Clone, derive_more::IsVariant, derive_more::Unwrap, derive_more::TryUnwrap, VariantName,
)]
pub enum ParameterValue {
    /// A sampler parameter
    Sampler(Sampler),
    // /// A texture parameter
    // Texture2D(Texture),
}

/// Public API
impl NativeMaterial {
    /// Creates a new native material from the given shader, with no keywords set
    pub(crate) fn new(shader: Arc<Shader>) -> Self {
        let mut new_self = Self {
            id: MaterialId::new(),
            parameters: map![],
            shader,
            keywords: map![],
            keywords_with_defaults: map![],
            compiled_shader_id: CompiledShaderId(0),
        };

        new_self.recalculate_compiled_shader_id();
        new_self.reset_parameter_values();
        new_self.recalculate_keywords_with_defaults();

        new_self
    }

    /// Returns the ID of this material
    #[inline(always)]
    pub(crate) const fn id(&self) -> MaterialId {
        self.id
    }

    /// Returns the ID of the compiled shader of this material
    #[inline(always)]
    pub(crate) const fn compiled_shader_id(&self) -> CompiledShaderId {
        self.compiled_shader_id
    }

    /// Returns the shader this material uses
    #[inline(always)]
    pub(crate) fn shader(&self) -> &Shader {
        &self.shader
    }

    /// Returns the keywords set on this material, including the values for default ones
    #[inline(always)]
    pub(crate) const fn get_keywords(&self) -> &HashMap<String, u64> {
        &self.keywords_with_defaults
    }

    /// Updates the shader this material uses.
    /// Removes all keywords not present on the new shader, but
    /// retains the keywords that are. Also resets all parameter values to [None].
    pub(crate) fn set_shader(&mut self, shader: Arc<Shader>) {
        // Remove all keywords not in the new shader
        self.keywords
            .retain(|k, _| shader.allowed_keywords.contains_key(k));

        self.shader = shader;
        self.recalculate_compiled_shader_id();
        self.reset_parameter_values();
        self.recalculate_keywords_with_defaults();
    }

    /// Sets or updates a keyword value on this material. If the keyword was not
    /// set before, it is set now. Otherwise it is simply updated
    pub(crate) fn set_keyword(&mut self, keyword: &str, value: u64) {
        if let Err(e) = Self::verify_single_keyword(keyword, value, &self.shader) {
            log::error!("Tried to set invalid keyword value, ignoring: {e}");
            return;
        }

        if let Some(cur_val) = self.keywords.get_mut(keyword) {
            *cur_val = value;
            return;
        }

        self.keywords.insert(keyword.to_owned(), value);
        self.recalculate_compiled_shader_id();
        self.recalculate_keywords_with_defaults();
    }

    /// Unsets a keyword. Equivalent to setting its value to `0`
    #[inline]
    pub(crate) fn unset_keyword(&mut self, keyword: &str) {
        self.set_keyword(keyword, 0);
    }

    pub(crate) fn set_parameter(&mut self, param: &str, value: Option<ParameterValue>) {
        let Some(cur_value) = self.parameters.get_mut(param) else {
            log::error!(
                "Parameter \"{param}\" does not exist on shader {}",
                self.shader.name
            );
            return;
        };

        let Some(new_value) = value else {
            *cur_value = None;
            return;
        };

        let param_descriptor = self
            .shader
            .parameters
            .get(param)
            .expect("Parameter should exist on the shader if it exists in the native material");

        if !Self::verify_parameter_type(param_descriptor, &new_value) {
            log::error!(
                "Incompatible value type for shader parameter. Expected {}, got {}",
                param_descriptor.ty,
                new_value.variant_name()
            );
            return;
        }

        *cur_value = Some(new_value);
    }

    #[inline]
    pub(crate) fn unset_parameter(&mut self, param: &str) {
        self.set_parameter(param, None);
    }
}

/// Private API
impl NativeMaterial {
    fn verify_parameter_type(
        parameter: &shader::ShaderParameter,
        new_value: &ParameterValue,
    ) -> bool {
        match parameter.ty {
            shader::ShaderParameterType::Sampler => new_value.is_sampler(),
            shader::ShaderParameterType::Texture2D => todo!(),
        }
    }

    /// Injects the default values for the missing keywords in `keywords`. A keyword is considered missing if
    /// it is part of the [allowed keywords](Shader::allowed_keywords) in `shader`, but is not
    /// in `keywords`
    ///
    /// NOTE: Does not verify that the already present keywords are correct. Calling [Self::verify_keywords]
    /// beforehand is recommended.
    fn inject_defaults_for_shader(shader: &Shader, keywords: &mut HashMap<String, u64>) {
        for allowed_keyword in shader.allowed_keywords.keys() {
            if !keywords.contains_key(allowed_keyword) {
                keywords.insert(allowed_keyword.clone(), 0);
            }
        }
    }

    /// Verifies that all keyword/value pairs in `to_verify` would be valid to set when using the given shader.
    fn verify_keywords<'a>(
        to_verify: &'a HashMap<String, u64>,
        shader: &Shader,
    ) -> Result<(), Box<VerifyErr<'a>>> {
        for (keyword, value) in to_verify.iter() {
            Self::verify_single_keyword(keyword.as_str(), *value, shader)?;
        }

        Ok(())
    }

    /// Verifies that a given keyword/value combination would be valid to set when using the given shader.
    fn verify_single_keyword<'a>(
        key: &'a str,
        value: u64,
        shader: &Shader,
    ) -> Result<(), Box<VerifyErr<'a>>> {
        if let Some(allowed_keyword_values) = shader.allowed_keywords.get(key) {
            if !allowed_keyword_values.contains(&value) {
                return Err(Box::new(VerifyErr::InvalidKeywordValue(
                    key,
                    value,
                    allowed_keyword_values.clone(),
                )));
            }
        } else {
            return Err(Box::new(VerifyErr::UnknownKeyword(key)));
        }

        Ok(())
    }

    fn recalculate_compiled_shader_id(&mut self) {
        let mut keywords = self.keywords.clone();
        Self::inject_defaults_for_shader(&self.shader, &mut keywords);

        self.compiled_shader_id = self.shader.create_compiled_shader_id(&keywords);
    }

    fn reset_parameter_values(&mut self) {
        self.parameters.clear();
        self.parameters
            .extend(self.shader.parameters.keys().map(|k| (k.clone(), None)));
    }

    fn recalculate_keywords_with_defaults(&mut self) {
        self.keywords_with_defaults.clear();
        self.keywords_with_defaults
            .extend(self.keywords.iter().map(|(k, v)| (k.clone(), *v)));

        Self::inject_defaults_for_shader(&self.shader, &mut self.keywords_with_defaults);
    }
}

/// Keywords on a material are not compatible with its shader
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum VerifyErr<'a> {
    /// A keyword is set on the material that is not present
    #[display("Keyword on material not known by the used shader: {}", _0)]
    UnknownKeyword(#[error(not(source))] &'a str),

    /// A keyword has a value that is not valid for the shader
    #[display("Invalid value for keyword {} with allowed range {}..={}: {}", _0, _2.start(), _2.end(), _1)]
    InvalidKeywordValue(&'a str, u64, RangeInclusive<u64>),
}

impl Clone for NativeMaterial {
    fn clone(&self) -> Self {
        Self {
            id: MaterialId::new(),
            shader: self.shader.clone(),
            keywords: self.keywords.clone(),
            keywords_with_defaults: self.keywords_with_defaults.clone(),
            compiled_shader_id: self.compiled_shader_id,
            parameters: self.parameters.clone(),
        }
    }
}
