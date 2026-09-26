#![doc = include_str!("../README.md")]

use core::error::Error;
use core::fmt::Write;
use core::num::NonZero;
use core::range::Range;
use core::range::RangeInclusive;
use core::str::FromStr;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use wutengine_assets::assets::shader::PrecompiledShader;
use wutengine_assets::assets::shader::ShaderHash;
use wutengine_util::JobQueue;

use crate::bindings::FindBindingsErr;
use crate::engine::LayoutErr;
use crate::preprocessor::Declarations;
use crate::preprocessor::KeywordDecl;
use crate::preprocessor::PreprocessErr;
use crate::vertex_inputs::FindVertexInputsErr;

pub mod bindings;
pub mod engine;
pub mod preprocessor;
pub mod vertex_inputs;

/// An error while compiling a shader
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum CompileErr {
    /// Preprocessing failed
    #[display("Error during preprocessing: {_0}")]
    Preprocess(PreprocessErr),

    /// The keyword values don't fit the declared keywords
    #[display("Invalid keywords: {_0}")]
    Keywords(KeywordErr),

    /// Failed to parse WGSL. Holds the rendered diagnostic, with source locations
    #[display("Failed to parse WGSL:\n{_0}")]
    #[from(skip)]
    Parse(#[error(not(source))] String),

    /// The parsed module is invalid. Holds the rendered diagnostic, with source locations
    #[display("Invalid shader:\n{_0}")]
    #[from(skip)]
    Validate(#[error(not(source))] String),

    /// Failed to find shader bindings
    #[display("Failed to find shader bindings: {_0}")]
    FindBindings(FindBindingsErr),

    /// Failed to find vertex inputs
    #[display("Failed to find shader vertex inputs: {_0}")]
    FindVertexInputs(FindVertexInputsErr),

    /// The bindings don't fit the engine's bind group layout
    #[display("{_0}")]
    Layout(LayoutErr),
}

/// Keyword values that don't fit a shader's `#keyword` declarations
#[derive(Debug, derive_more::Error, derive_more::Display)]
pub enum KeywordErr {
    /// The keyword isn't declared by the shader
    #[display("Keyword `{_0}` is not declared by the shader")]
    Undeclared(#[error(not(source))] String),

    /// The value is outside the declared range
    #[display("Value {value} of keyword `{keyword}` is outside its allowed range {allowed:?}")]
    OutOfRange {
        /// The keyword
        keyword: String,

        /// The given value
        value: u64,

        /// The declared range
        allowed: RangeInclusive<u64>,
    },
}

/// The variant-independent information of a shader source: its name, declared keywords and source hash
#[derive(Debug, Clone)]
pub struct ShaderInfo {
    /// The `#name` and `#keyword` directives
    declarations: Declarations,

    /// Hash of the full source text
    source_hash: u128,
}

impl FromStr for ShaderInfo {
    type Err = PreprocessErr;

    /// Reads the declarations of a shader source
    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            declarations: preprocessor::declarations(source)?,
            source_hash: twox_hash::xxhash3_128::Hasher::oneshot(source.as_bytes()),
        })
    }
}

impl ShaderInfo {
    /// The shader name
    #[inline]
    pub fn name(&self) -> &str {
        &self.declarations.name
    }

    /// The declared keywords
    #[inline]
    pub fn keywords(&self) -> &BTreeMap<Arc<str>, KeywordDecl> {
        &self.declarations.keywords
    }

    /// Returns the variant for the given keyword values. Keywords that aren't given get their default value.
    pub fn variant<K: AsRef<str>>(
        &self,
        keywords: impl IntoIterator<Item = (K, u64)>,
    ) -> Result<Variant, KeywordErr> {
        let declared = &self.declarations.keywords;

        let mut values: BTreeMap<Arc<str>, u64> = declared
            .iter()
            .map(|(name, decl)| (name.clone(), decl.default))
            .collect();

        for (keyword, value) in keywords {
            let keyword = keyword.as_ref();

            let (name, decl) = declared
                .get_key_value(keyword)
                .ok_or_else(|| KeywordErr::Undeclared(keyword.to_owned()))?;

            if !decl.allowed.contains(&value) {
                return Err(KeywordErr::OutOfRange {
                    keyword: keyword.to_owned(),
                    value,
                    allowed: decl.allowed,
                });
            }

            values.insert(name.clone(), value);
        }

        let mut hash_input = format!("{:032x}", self.source_hash);

        for (keyword, value) in &values {
            write!(hash_input, ";{keyword}:{value}").expect("Writing to a String can't fail");
        }

        Ok(Variant {
            name: self.declarations.name.clone(),
            // TODO: Imported sources are not part of the hash, so editing an import doesn't invalidate precompiled
            // variants. Hash the resolved imports once the resolver can report them.
            hash: ShaderHash(twox_hash::xxhash3_128::Hasher::oneshot_with_seed(
                0x0299_450944,
                hash_input.as_bytes(),
            )),
            keywords: values,
        })
    }
}

/// One variant of a shader: a value for every declared keyword. Created with [`ShaderInfo::variant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// The shader name
    name: String,

    /// The hash identifying this variant
    hash: ShaderHash,

    /// The value of every declared keyword
    keywords: BTreeMap<Arc<str>, u64>,
}

impl Variant {
    /// The hash identifying this variant
    #[inline]
    pub fn hash(&self) -> ShaderHash {
        self.hash
    }

    /// The value of every declared keyword
    #[inline]
    pub fn keywords(&self) -> &BTreeMap<Arc<str>, u64> {
        &self.keywords
    }
}

/// Compiles one variant of `source`. `variant` must come from the [`ShaderInfo`] of this same source. Imports
/// other than the built-in [`engine::IMPORT_NAME`] are resolved with `shader_resolver`.
pub fn compile(
    source: &str,
    variant: &Variant,
    shader_resolver: Option<&dyn ShaderResolver>,
) -> Result<PrecompiledShader, CompileErr> {
    profiling::function_scope!();

    log::info!("Compiling `{}` ({})", variant.name, variant.hash);

    let preprocessed =
        preprocessor::preprocess(source, &variant.keywords, &EngineResolver(shader_resolver))?;

    log::debug!("Preprocessing result:\n{preprocessed}");

    let module = {
        profiling::scope!("Naga parse");

        let mut naga_frontend =
            naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
                parse_doc_comments: true,
                capabilities: naga::valid::Capabilities::default(),
            });

        naga_frontend
            .parse(&preprocessed)
            .map_err(|e| CompileErr::Parse(e.emit_to_string(&preprocessed)))?
    };

    let module_info = {
        profiling::scope!("Naga validate");

        // All capabilities: the device the shader runs on is unknown here, and wgpu validates against its actual
        // capabilities when creating the shader module.
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .map_err(|e| CompileErr::Validate(e.emit_to_string(&preprocessed)))?
    };

    // Only used bindings, so an import like the engine's doesn't add bind groups the shader never touches
    let is_used = |global: naga::Handle<naga::GlobalVariable>| {
        (0..module.entry_points.len()).any(|i| !module_info.get_entry_point(i)[global].is_empty())
    };

    let (bindings, vertex_inputs) = rayon::join(
        || bindings::find_bindings(&module, is_used),
        || vertex_inputs::find_vertex_inputs(&module),
    );

    let bindings = bindings?;
    let vertex_inputs = vertex_inputs?;

    engine::check_layout(&bindings)?;

    Ok(PrecompiledShader {
        hash: variant.hash,
        name: variant.name.clone(),
        module: Box::new(module),
        bindings,
        vertex_inputs,
    })
}

/// Configuration for a [multi-compile job](compile_multiple)
#[derive(Debug)]
pub struct MultiConfig {
    /// The compiled buffer size. Will buffer a maximum of this amount of compiled shaders, before waiting on the receiving channel to read some output
    pub buf_size: NonZero<usize>,

    /// The keyword ranges to compile
    pub keywords: HashMap<String, Range<u64>>,

    /// The [`ShaderResolver`] to use
    pub shader_resolver: Option<Arc<dyn ShaderResolver>>,
}

/// Compiles multiple permutations of the input source. All compilation results are sent to a channel,
/// for which the receiving end is returned
pub fn compile_multiple(
    input: &str,
    config: MultiConfig,
) -> Receiver<Result<PrecompiledShader, CompileErr>> {
    log::info!("Starting multi-compile job");

    let (send, recv) = std::sync::mpsc::sync_channel(config.buf_size.get());

    let input: Arc<str> = Arc::from(input);
    let keywords: Vec<(Arc<str>, Range<u64>)> = config
        .keywords
        .iter()
        .map(|(k, v)| (Arc::from(k.as_str()), *v))
        .collect();

    let resolver = config.shader_resolver;

    rayon::spawn(move || {
        let info = match input.parse::<ShaderInfo>() {
            Ok(info) => Arc::new(info),
            Err(e) => {
                _ = send.send(Err(e.into()));
                return;
            }
        };

        let queue = JobQueue::new(config.buf_size);

        for combination in KeywordCombinations::new(&keywords) {
            let Ok(token) = queue.issue_job() else {
                return;
            };

            let input = input.clone();
            let info = info.clone();
            let resolver = resolver.clone();
            let send = send.clone();

            rayon::spawn(move || {
                log::debug!("Compiling with keywords: {combination:#?}");

                let result = info
                    .variant(combination)
                    .map_err(CompileErr::from)
                    .and_then(|variant| {
                        compile(&input, &variant, resolver.as_deref().map(|r| r as _))
                    });

                let Ok(()) = send.send(result) else {
                    log::error!("Shader result channel closed?");
                    token.cancel_job_queue();
                    return;
                };

                drop(token);
            });
        }
    });

    recv
}

/// Iterator that returns all combinations of a set of keywords and ranges
struct KeywordCombinations<'a> {
    /// The ranges to go through
    ranges: &'a [(Arc<str>, Range<u64>)],

    /// The current value
    current: Vec<u64>,

    /// Whether we're done
    finished: bool,
}

impl<'a> KeywordCombinations<'a> {
    /// A new combinations iterator
    fn new(ranges: &'a [(Arc<str>, Range<u64>)]) -> Self {
        let current = ranges.iter().map(|(_, range)| range.start).collect();

        Self {
            ranges,
            current,
            finished: ranges.iter().any(|(_, range)| range.is_empty()),
        }
    }
}

impl Iterator for KeywordCombinations<'_> {
    type Item = HashMap<Arc<str>, u64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Construct the next combination.
        let result = self
            .ranges
            .iter()
            .zip(&self.current)
            .map(|((name, _), &value)| (name.clone(), value))
            .collect();

        // Increment the "odometer", starting at the last range.
        for i in (0..self.ranges.len()).rev() {
            let range = &self.ranges[i].1;

            self.current[i] += 1;

            if self.current[i] < range.end {
                // No carry needed.
                return Some(result);
            }

            // This position overflowed, so reset it and carry.
            self.current[i] = range.start;
        }

        // Everything overflowed, so we're done.
        self.finished = true;

        Some(result)
    }
}

/// A type that can resolve shader source by name
pub trait ShaderResolver: core::fmt::Debug + Send + Sync {
    /// For a given shader name, returns the source code.
    fn find_by_name(&self, shader_name: &str) -> Result<String, Box<dyn Error + Send>>;
}

/// Resolves the built-in engine import, and passes everything else to the wrapped resolver
#[derive(Debug)]
struct EngineResolver<'a>(Option<&'a dyn ShaderResolver>);

impl ShaderResolver for EngineResolver<'_> {
    fn find_by_name(&self, shader_name: &str) -> Result<String, Box<dyn Error + Send>> {
        if shader_name == engine::IMPORT_NAME {
            return Ok(engine::import_source().to_owned());
        }

        self.0
            .unwrap_or(&UnsupportedResolver)
            .find_by_name(shader_name)
    }
}

/// Simple internal shader resolver that always errors
#[derive(Debug)]
struct UnsupportedResolver;

impl ShaderResolver for UnsupportedResolver {
    fn find_by_name(&self, shader_name: &str) -> Result<String, Box<dyn Error + Send>> {
        #[derive(Debug, derive_more::Error, derive_more::Display)]
        #[display("No resolver was given")]
        struct Unsupported;

        _ = shader_name;

        Err(Box::new(Unsupported))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SOURCE: &str = "#name \"Test\"\n#keyword A 0..3\n#keyword B\n";

    /// Tests that omitted keywords hash like their defaults, to prevent a precompiled variant from being missed
    /// when a material leaves a keyword unset
    #[test]
    fn variant_defaults() {
        let info = SOURCE.parse::<ShaderInfo>().unwrap();

        let implicit = info.variant::<&str>([]).unwrap();
        let explicit = info.variant([("A", 0), ("B", 0)]).unwrap();
        let other = info.variant([("A", 2)]).unwrap();

        assert_eq!(implicit, explicit);
        assert_ne!(implicit.hash(), other.hash());
    }

    /// Tests that the hash covers the source, to prevent an edited shader from matching stale precompiled variants
    #[test]
    fn variant_hash_includes_source() {
        let a = SOURCE
            .parse::<ShaderInfo>()
            .unwrap()
            .variant::<&str>([])
            .unwrap();
        let b = format!("{SOURCE}fn f() {{}}")
            .parse::<ShaderInfo>()
            .unwrap()
            .variant::<&str>([])
            .unwrap();

        assert_ne!(a.hash(), b.hash());
    }

    /// Tests keyword validation, to prevent typos and out-of-range values from compiling a wrong variant
    #[test]
    fn variant_errors() {
        let info = SOURCE.parse::<ShaderInfo>().unwrap();

        assert!(matches!(
            info.variant([("C", 0)]),
            Err(KeywordErr::Undeclared(_))
        ));
        assert!(matches!(
            info.variant([("A", 3)]),
            Err(KeywordErr::OutOfRange { .. })
        ));
    }

    /// Tests that every combination is produced once, to prevent the multi-compile from skipping variants
    #[test]
    fn keyword_combinations() {
        let ranges = [
            (Arc::from("A"), Range { start: 0, end: 2 }),
            (Arc::from("B"), Range { start: 5, end: 8 }),
        ];

        let all: Vec<_> = KeywordCombinations::new(&ranges).collect();

        assert_eq!(6, all.len());
        assert!(all.iter().any(|c| c["A"] == 1 && c["B"] == 7));
        assert_eq!(
            0,
            KeywordCombinations::new(&[(Arc::from("A"), Range { start: 1, end: 1 })]).count()
        );
    }

    /// Tests a full compile against the engine import, to prevent regressions in used-binding filtering and the
    /// reserved group check
    #[test]
    fn compile_engine_layout() {
        let source = "#name \"T\"\n#import \"wutengine\"\n\
            @group(WUTENGINE_MATERIAL_GROUP) @binding(0) var<uniform> tint: vec4f;\n\
            @group(WUTENGINE_MATERIAL_GROUP) @binding(1) var<uniform> unused: vec4f;\n\
            fn get_tint() -> vec4f { return tint; }\n\
            @vertex fn vs(@location(0) position: vec3f) -> @builtin(position) vec4f {\n\
                return instance_params.mvp * vec4f(position, 1.0) * get_tint();\n\
            }\n";

        let info = source.parse::<ShaderInfo>().unwrap();
        let compiled = compile(source, &info.variant::<&str>([]).unwrap(), None).unwrap();

        let names: Vec<&str> = compiled.bindings.iter().map(|b| b.name.as_str()).collect();

        // Used through a helper function, unused, and only used by the engine import
        assert!(names.contains(&"tint"));
        assert!(!names.contains(&"unused"));
        assert!(!names.contains(&"camera_params"));

        let bad = source.replace("WUTENGINE_MATERIAL_GROUP", "WUTENGINE_CAMERA_GROUP");
        let info = bad.parse::<ShaderInfo>().unwrap();

        assert!(matches!(
            compile(&bad, &info.variant::<&str>([]).unwrap(), None),
            Err(CompileErr::Layout(LayoutErr::Reserved(_)))
        ));
    }
}
