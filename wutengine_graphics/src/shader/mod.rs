use core::hash::Hash;
use std::collections::HashSet;

pub mod builtins;

#[derive(Debug, Clone)]
pub struct Shader {
    pub source: ShaderSource,
    pub available_keywords: Vec<String>,
}

impl Shader {
    pub fn make_variant(
        &self,
        set_keywords: impl IntoIterator<Item = impl Into<String>>,
    ) -> ShaderVariant {
        let mut variant = ShaderVariant {
            source: self.source.clone(),
            set_keywords: HashSet::new(),
        };

        for kw in set_keywords {
            let keyword_string = kw.into();

            if cfg!(debug_assertions) && !self.available_keywords.contains(&keyword_string) {
                log::warn!(
                    "Keyword {} not found as an available keyword for shader {:#?}",
                    keyword_string,
                    self.source
                )
            }

            variant.set_keywords.insert(keyword_string);
        }

        variant
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderVariant {
    pub source: ShaderSource,
    pub set_keywords: HashSet<String>,
}

impl Hash for ShaderVariant {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);

        let mut as_vec: Vec<&String> = self.set_keywords.iter().collect();

        as_vec.sort();

        for keyword in as_vec {
            keyword.hash(state);
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShaderKeywordType {}

#[derive(Debug, Clone)]
pub enum ShaderKeywordValue {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ShaderSource {
    Builtin { identifier: &'static str },
}
