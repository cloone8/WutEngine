use core::fmt::Display;
use std::borrow::Cow;
use std::collections::HashMap;

use md5::{Digest, Md5};

use crate::shader::is_valid_keyword;

/// The ID of a [Shader]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderId {
    ident: String,
    keywords: Option<u128>,
}

impl ShaderId {
    pub fn new_no_keywords(ident: impl Into<String>) -> Self {
        Self {
            ident: ident.into(),
            keywords: None,
        }
    }

    /// Generates a new [ShaderId] with the given keyword values and raw identifier
    pub fn new_with_keywords(ident: impl Into<String>, keywords: &HashMap<String, u32>) -> Self {
        if keywords.is_empty() {
            return Self::new_no_keywords(ident);
        }

        for keyword in keywords.keys() {
            assert!(is_valid_keyword(keyword));
        }

        let mut keyword_list = keywords.iter().collect::<Vec<_>>();
        keyword_list.sort_unstable_by_key(|kw| kw.0);

        let mut hasher = Md5::new();

        for (kw, val) in keyword_list {
            hasher.update(kw);
            hasher.update(val.to_le_bytes());
        }

        let result = u128::from_ne_bytes(hasher.finalize().into());

        Self {
            ident: ident.into(),
            keywords: Some(result),
        }
    }

    pub fn ident(&self) -> &String {
        &self.ident
    }

    pub fn keyword_hash(&self) -> Option<u128> {
        self.keywords
    }

    pub fn without_keywords(&self) -> Cow<'_, Self> {
        if self.keywords.is_none() {
            Cow::Borrowed(self)
        } else {
            Cow::Owned(ShaderId::new_no_keywords(self.ident.clone()))
        }
    }
}

impl Display for ShaderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.keywords {
            Some(kws) => format!("{}-{:32x}", self.ident, kws).fmt(f),
            None => self.ident.fmt(f),
        }
    }
}
