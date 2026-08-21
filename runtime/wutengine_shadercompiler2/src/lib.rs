#![doc = include_str!("../README.md")]

use core::error::Error;
use core::num::NonZero;
use core::range::Range;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use wutengine_assets::assets::shader::PrecompiledShader;
use wutengine_assets::assets::shader::ShaderHash;
use wutengine_util::JobQueue;

use crate::preprocessor::PreprocessErr;

pub mod parameters;
pub mod preprocessor;

/// An error while compiling a shader
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum CompileErr {
    /// Preprocessing failed
    #[display("Error during preprocessing: {_0}")]
    Preprocess(PreprocessErr),

    /// Failed to parse WGSL
    #[display("Failed to parse WGSL: {_0}")]
    CompileIR(Box<naga::front::wgsl::ParseError>),
}

/// A compilation job configuration
#[derive(Debug)]
pub struct Config<R> {
    /// Enabled keyword values
    pub keywords: HashMap<Arc<str>, u64>,

    /// The [`ShaderResolver`] to use
    pub shader_resolver: Option<R>,
}

/// Compiles the given input using the provided config
pub fn compile<R: ShaderResolver>(
    input: &str,
    config: &Config<R>,
) -> Result<PrecompiledShader, CompileErr> {
    profiling::function_scope!();

    log::info!("Compiling input");

    let preprocess_result = preprocessor::preprocess(
        input,
        config.keywords.clone(),
        config
            .shader_resolver
            .as_ref()
            .map_or(&UnsupportedResolver, |r| r as &dyn ShaderResolver),
    )?;

    let hash = hash_shader(
        &preprocess_result.name,
        config.keywords.iter().map(|(k, v)| (k, *v)),
    );

    log::info!("Preprocessed `{}` ({})", preprocess_result.name, hash);

    log::debug!("Preprocessing result:\n{}", preprocess_result.source);

    log::debug!("Parsing WGSL to Naga IR");

    let module = {
        profiling::scope!("Naga parse");

        let mut naga_frontend =
            naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
                parse_doc_comments: true,
                capabilities: naga::valid::Capabilities::default(),
            });

        naga_frontend
            .parse(&preprocess_result.source)
            .map_err(Box::new)
            .map(Box::new)?
    };

    let parameters = parameters::find_parameters(&module);

    Ok(PrecompiledShader {
        hash,
        module,
        parameters,
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

    let resolver = config
        .shader_resolver
        .unwrap_or(Arc::new(UnsupportedResolver));

    rayon::spawn(move || {
        let queue = JobQueue::new(config.buf_size);

        for combination in KeywordCombinations::new(&keywords) {
            let Ok(token) = queue.issue_job() else {
                return;
            };

            let input = input.clone();
            let resolver = resolver.clone();
            let send = send.clone();

            rayon::spawn(move || {
                log::debug!("Compiling with keywords: {combination:#?}");

                let result = compile(
                    &input,
                    &Config {
                        keywords: combination,
                        shader_resolver: Some(resolver),
                    },
                );

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

impl ShaderResolver for Arc<dyn ShaderResolver> {
    #[inline]
    fn find_by_name(&self, shader_name: &str) -> Result<String, Box<dyn Error + Send>> {
        self.as_ref().find_by_name(shader_name)
    }
}

/// Produces a unique hash
pub fn hash_shader<'a, S>(
    shader_name: &str,
    keywords: impl IntoIterator<Item = (&'a S, u64)>,
) -> ShaderHash
where
    S: AsRef<str> + 'a,
{
    const KEY_SEPERATOR: &str = ";";
    const VAL_SEPERATOR: &str = ":";

    profiling::function_scope!();

    let shader_name = shader_name.trim();

    debug_assert!(
        !shader_name.contains(KEY_SEPERATOR) && !shader_name.contains(VAL_SEPERATOR),
        "Invalid characters in shader name: {shader_name}"
    );

    let mut collected = keywords
        .into_iter()
        .map(|(k, v)| {
            let key = k.as_ref().trim();

            debug_assert!(
                !key.contains(KEY_SEPERATOR) && !key.contains(VAL_SEPERATOR),
                "Invalid characters in keyword: {key}"
            );

            (key, v)
        })
        .collect::<Vec<_>>();

    collected.sort_unstable_by_key(|x| x.0);

    let all_keywords = collected
        .into_iter()
        .map(|(keyword, value)| format!("{keyword}{VAL_SEPERATOR}{value}"))
        .collect::<Vec<_>>()
        .join(KEY_SEPERATOR);

    let total_string = format!("{shader_name}{KEY_SEPERATOR}{all_keywords}");

    ShaderHash(twox_hash::xxhash3_128::Hasher::oneshot_with_seed(
        0x0299_450944,
        total_string.as_bytes(),
    ))
}
