#![doc = include_str!("../README.md")]

use std::collections::HashMap;

use naga::front::wgsl::Options;
use naga::keywords;

/// Input for a shader compilation job
#[derive(Debug, Clone)]
pub struct Input<'a> {
    /// The identifier of the source code.
    pub source_id: &'a str,

    /// The original WGSL source code
    pub source: &'a str,

    /// The activated keywords for this compile job
    pub active_keywords: &'a HashMap<String, u64>,
}

/// Output for a shader compilation job
pub struct Output {
    /// The succesfully compiled [naga] module
    pub module: Box<naga::Module>,

    /// The hash of the source shader identifier
    pub source_id_hash: u64,

    /// The hash of the active keywords when compiling
    pub keyword_hash: u64,
}

/// An error during shader compilation
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum Error {
    /// Paring error
    ParseError(naga::front::wgsl::ParseError),
}

/// Compile a single input shader
pub fn compile<H: ShaderHasher>(input: Input) -> Result<Output, Error> {
    let mut opts = Options::new();
    opts.parse_doc_comments = true;

    let mut frontend = naga::front::wgsl::Frontend::new_with_options(opts);

    let compiled = Box::new(frontend.parse(input.source)?);

    Ok(Output {
        module: compiled,
        source_id_hash: H::hash_source_id(input.source_id),
        keyword_hash: H::hash_keywords(input.active_keywords),
    })
}

/// An implementation that provides deterministic hashes for a shader compilation
pub trait ShaderHasher {
    /// Converts the string ID of a shader to a hash value
    fn hash_source_id(id: &str) -> u64;
    /// Converts a map of keyword names and value to a single hash value
    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64;
}
