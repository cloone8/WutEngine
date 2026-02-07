#![doc = include_str!("../README.md")]

use core::fmt::Display;
use core::hash::Hash;
use std::collections::HashMap;

use naga::front::wgsl::Options;
use smallvec::SmallVec;

/// Input for a shader compilation job
#[derive(Debug)]
pub struct Input<'a, Id> {
    /// The identifier of the source code.
    pub source_id: Id,

    /// The original WGSL source code
    pub source: &'a str,

    /// The activated keywords for this compile job
    pub active_keywords: &'a HashMap<String, u64>,

    /// All possible bindings for the source shader
    pub all_bindings: &'a [Binding],
}

/// Output for a shader compilation job
#[derive(Debug)]
pub struct Output {
    /// The succesfully compiled [naga] module
    pub module: naga::Module,

    /// The hash of the source shader identifier
    pub source_id_hash: u64,

    /// The hash of the active keywords when compiling
    pub keyword_hash: u64,

    /// The bindings that were not compiled out during keyword resolution
    pub remaining_bindings: SmallVec<[Binding; 32]>,
}

/// A resource binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Binding {
    /// Bind group
    pub group: u32,

    /// Binding within the group
    pub binding: u32,
}

impl Hash for Binding {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let as_64: u64 = ((self.group as u64) << 32) | (self.binding as u64);

        state.write_u64(as_64);
    }
}

impl nohash_hasher::IsEnabled for Binding {}

impl From<(u32, u32)> for Binding {
    #[inline]
    fn from(value: (u32, u32)) -> Self {
        Self {
            group: value.0,
            binding: value.1,
        }
    }
}

impl Display for Binding {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Binding(group: {}, binding: {})",
            self.group, self.binding
        )
    }
}

/// An error during shader compilation
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum Error {
    /// Paring error
    ParseError(naga::front::wgsl::ParseError),

    /// An unknown binding was encountered after compilation
    #[display("A binding was encountered that was not declared: {_0}")]
    #[from(skip)]
    UnknownBinding(#[error(not(source))] Binding),

    /// A duplicate binding was encountered after compilation
    #[display("A duplicate binding was encountered after compilation: {_0}")]
    #[from(skip)]
    DuplicateBinding(#[error(not(source))] Binding),
}

/// Compile a single input shader
pub fn compile<Id, H: ShaderHasher<Id>>(input: Input<Id>) -> Result<Box<Output>, Error> {
    profiling::function_scope!();

    let mut opts = Options::new();
    opts.parse_doc_comments = true;

    let mut frontend = naga::front::wgsl::Frontend::new_with_options(opts);

    let compiled = frontend.parse(input.source)?;

    Ok(Box::new(Output {
        remaining_bindings: detect_bind_groups(input.all_bindings, &compiled)?,
        module: compiled,
        source_id_hash: H::hash_source_id(input.source_id),
        keyword_hash: H::hash_keywords(input.active_keywords),
    }))
}

/// Finds the bind groups in the compiled module that have not been stripped during compilation
fn detect_bind_groups<const N: usize>(
    all_bindings: &[Binding],
    module: &naga::Module,
) -> Result<SmallVec<[Binding; N]>, Error> {
    profiling::function_scope!();

    let mut bind_groups = SmallVec::new_const();

    for (_, global_var) in module.global_variables.iter() {
        let Some(binding) = global_var.binding else {
            continue;
        };

        let as_binding = Binding {
            group: binding.group,
            binding: binding.binding,
        };

        if all_bindings.contains(&as_binding) {
            if bind_groups.contains(&as_binding) {
                return Err(Error::DuplicateBinding(as_binding));
            }

            bind_groups.push(as_binding);
        } else {
            return Err(Error::UnknownBinding(as_binding));
        }
    }

    Ok(bind_groups)
}

/// An implementation that provides deterministic hashes for a shader compilation
pub trait ShaderHasher<Id> {
    /// Converts the string ID of a shader to a hash value
    fn hash_source_id(id: Id) -> u64;

    /// Converts a map of keyword names and value to a single hash value
    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64;
}
