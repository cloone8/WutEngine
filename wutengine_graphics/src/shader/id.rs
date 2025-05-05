use core::fmt::Display;
use std::borrow::Cow;
use std::iter::IntoIterator;
use std::vec::Vec;

use md5::{Digest, Md5};

use crate::shader::is_valid_keyword;

/// The ID of a [Shader]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderVariantId {
    /// The main identifier of the shader
    shader_ident: String,

    /// The state of the keywords of this variant of the main shader
    keywords: ShaderKeywordSet,
}

impl ShaderVariantId {
    /// Creates a new [ShaderVariantId] for the shader with the given identifier, with no keywords set
    pub fn new_no_keywords(ident: impl Into<String>) -> Self {
        Self {
            shader_ident: ident.into(),
            keywords: ShaderKeywordSet::new(),
        }
    }

    /// Generates a new [ShaderVariantId] with the given keyword values and raw identifier
    pub fn new_with_keywords(
        ident: impl Into<String>,
        keywords: impl IntoIterator<Item = (String, u32)>,
    ) -> Self {
        let keywords: Vec<(String, u32)> = keywords.into_iter().collect();

        if keywords.is_empty() {
            return Self::new_no_keywords(ident);
        }

        for keyword in keywords.iter().map(|(kw, _)| kw) {
            assert!(is_valid_keyword(keyword));
        }

        let mut new_id = Self {
            shader_ident: ident.into(),
            keywords: ShaderKeywordSet::new(),
        };

        new_id.keywords.set_keywords(keywords);

        new_id
    }

    /// Returns the main (non-variant) identifier of this shader
    pub fn ident(&self) -> &String {
        &self.shader_ident
    }

    /// Returns the set of keywords active on this variant
    pub fn keywords(&self) -> &ShaderKeywordSet {
        &self.keywords
    }

    /// Returns a clone of this [ShaderVariantId] without any of its
    /// keywords set
    pub fn without_keywords(&self) -> Cow<'_, Self> {
        if self.keywords.is_empty() {
            Cow::Borrowed(self)
        } else {
            Cow::Owned(ShaderVariantId::new_no_keywords(self.shader_ident.clone()))
        }
    }
}

impl Display for ShaderVariantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.keywords.is_empty() {
            true => self.shader_ident.fmt(f),
            false => format!("{}-{}", self.shader_ident, self.keywords).fmt(f),
        }
    }
}

/// A set of keywords on a shader
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ShaderKeywordSet {
    store: Vec<(String, u32)>,
}

impl ShaderKeywordSet {
    /// Creates a new, empty keyword set
    pub const fn new() -> Self {
        Self { store: Vec::new() }
    }

    /// Returns whether this set of keywords is empty
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Sets the given keyword to the given value, overwriting the old value (if present)
    pub fn set_keyword(&mut self, keyword: impl Into<String>, value: u32) {
        let kw_str = keyword.into();

        match self.store.binary_search_by_key(&&kw_str, |(kw, _)| kw) {
            Ok(present_idx) => {
                self.store[present_idx] = (kw_str, value);
            }
            Err(target_idx) => {
                self.store.insert(target_idx, (kw_str, value));
            }
        }
    }

    /// Sets the given keywords to the given values, overwriting any old ones. If duplicate
    /// keywords are given, later iterator values overwrite earlier ones
    pub fn set_keywords(&mut self, keywords: impl IntoIterator<Item = (String, u32)>) {
        //TODO: Optimize
        for (kw, val) in keywords {
            self.set_keyword(kw, val);
        }
    }

    /// Returns an iterator over the keywords in this set
    pub fn keywords(&self) -> impl IntoIterator<Item = &(String, u32)> {
        &self.store
    }

    /// Computes the (stable) hash for this set of keywords.
    /// If no keywords are set, always returns `0`
    pub fn compute_hash(&self) -> u128 {
        if self.store.is_empty() {
            return 0;
        }

        let mut hasher = Md5::new();

        for (kw, val) in &self.store {
            hasher.update(kw);
            hasher.update(val.to_ne_bytes());
        }

        u128::from_ne_bytes(hasher.finalize().into())
    }
}

impl IntoIterator for ShaderKeywordSet {
    type Item = (String, u32);

    type IntoIter = <Vec<(std::string::String, u32)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.store.into_iter()
    }
}

impl<'a> IntoIterator for &'a ShaderKeywordSet {
    type Item = &'a (String, u32);

    type IntoIter = <&'a Vec<(std::string::String, u32)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.store.iter()
    }
}

impl Display for ShaderKeywordSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.store.is_empty() {
            true => "".fmt(f),
            false => write!(f, "{:32x}", self.compute_hash()),
        }
    }
}
