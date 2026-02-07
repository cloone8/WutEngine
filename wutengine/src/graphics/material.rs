//! Material functionality

use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::sync::Arc;

use wutengine_util_macro::unique_id_type64;

use crate::map;

use super::shaders::{CompiledShader, Shader};

unique_id_type64! {
    /// The unique identifier for a [NativeMaterial]
    pub(crate) MaterialId
}

/// A material used for rendering
#[derive(Debug)]
pub(crate) struct NativeMaterial {
    id: MaterialId,

    /// The shader this material uses
    pub(crate) shader: Arc<Shader>,

    /// The overridden keywords in this material.
    /// Any keywords present in [Self::shader] but not present in this map
    /// are set to `0`
    pub(crate) keywords: HashMap<String, u64>,

    /// The compiled shader with the configuration set on this material
    compiled_shader: Option<Arc<CompiledShader>>,
}

impl NativeMaterial {
    /// Returns the ID of this material
    #[inline(always)]
    pub(crate) const fn id(&self) -> MaterialId {
        self.id
    }
    /// Creates a new native material from the given shader, with no keywords set
    pub(crate) fn new(shader: Arc<Shader>) -> Self {
        Self {
            id: MaterialId::new(),
            shader,
            keywords: map![],
            compiled_shader: None,
        }
    }

    /// Updates the shader this material uses.
    /// Removes all keywords not present on the new shader, but
    /// retains the keywords that are
    pub(crate) fn set_shader(&mut self, shader: Arc<Shader>) {
        // Remove all keywords not in the new shader
        self.keywords
            .retain(|k, _| shader.allowed_keywords.contains_key(k));

        self.shader = shader;

        self.compiled_shader = None;
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

        self.compiled_shader = None;
    }

    /// Unsets a keyword. Equivalent to setting its value to `0`
    pub(crate) fn unset_keyword(&mut self, keyword: &str) {
        self.set_keyword(keyword, 0);
    }

    /// Injects the default values for the missing keywords in `keywords`. A keyword is considered missing if
    /// it is part of the [allowed keywords](Shader::allowed_keywords) in `shader`, but is not
    /// in `keywords`
    ///
    /// NOTE: Does not verify that the already present keywords are correct. Calling [Self::verify_keywords]
    /// beforehand is recommended.
    pub(crate) fn inject_defaults_for_shader(shader: &Shader, keywords: &mut HashMap<String, u64>) {
        for allowed_keyword in shader.allowed_keywords.keys() {
            if !keywords.contains_key(allowed_keyword) {
                keywords.insert(allowed_keyword.clone(), 0);
            }
        }
    }

    /// Verifies that all keyword/value pairs in `to_verify` would be valid to set when using the given shader.
    pub(crate) fn verify_keywords<'a>(
        to_verify: &'a HashMap<String, u64>,
        shader: &Shader,
    ) -> Result<(), VerifyErr<'a>> {
        for (keyword, value) in to_verify.iter() {
            Self::verify_single_keyword(keyword.as_str(), *value, shader)?;
        }

        Ok(())
    }

    /// Verifies that a given keyword/value combination would be valid to set when using the given shader.
    pub(crate) fn verify_single_keyword<'a>(
        key: &'a str,
        value: u64,
        shader: &Shader,
    ) -> Result<(), VerifyErr<'a>> {
        if let Some(allowed_keyword_values) = shader.allowed_keywords.get(key) {
            if !allowed_keyword_values.contains(&value) {
                return Err(VerifyErr::InvalidKeywordValue(
                    key,
                    value,
                    allowed_keyword_values.clone(),
                ));
            }
        } else {
            return Err(VerifyErr::UnknownKeyword(key));
        }

        Ok(())
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
            compiled_shader: self.compiled_shader.clone(),
        }
    }
}
