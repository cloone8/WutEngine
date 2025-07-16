//! WutEngine Shader Compiler library

use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::time::Instant;

use rayon::Scope;
use thiserror::Error;

use crate::preprocessor::PreprocessErr;

pub mod compiler;
pub mod preprocessor;

/// A value given to a shader keyword. Can be one specific value, or a range of values
#[derive(Debug, Clone)]
pub enum KeywordValue {
    /// One value
    Single(i64),

    /// A range of values
    Range(RangeInclusive<i64>),
}

impl KeywordValue {
    /// Flattens this value to an array of values. Returns a single element array
    /// for single value keywords, and the entire value range for range valued keywords
    fn flatten<K>(&self, key: K) -> Vec<(K, i64)>
    where
        K: core::hash::Hash + Eq + Clone,
    {
        match self {
            KeywordValue::Single(single) => vec![(key, *single)],
            KeywordValue::Range(range) => range.clone().map(|i| (key.clone(), i)).collect(),
        }
    }
}

/// How far in the compilation progress to take the shader
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompileStage {
    /// Preprocess only
    Preprocess = 0,

    /// Compile the full shader
    Full,
}

/// The output of shader compilation
#[derive(Debug, Clone)]
pub enum ShaderOutput<'a> {
    /// A preprocessed shader
    Preprocessed {
        /// WGSL source code
        source: String,

        /// Hash of the used keyword values
        keyword_hash: u128,

        /// Keywords used
        keywords: HashMap<&'a str, i64>,
    },

    /// A fully compiled WutEngine shader
    Compiled {
        /// The [naga] IR source
        source: Box<naga::Module>,

        /// Hash of the used keyword values
        keyword_hash: u128,

        /// Keywords used
        keywords: HashMap<&'a str, i64>,
    },
}

fn compile_job_recursive<'a, 'scope>(
    scope: &Scope<'scope>,
    my_keywords: Vec<(&'a str, i64)>,
    keywords: &[Vec<(&'a str, i64)>],
    callback: impl Fn(HashMap<&'a str, i64>) + Send + Sync + 'scope + Clone,
) where
    'a: 'scope,
{
    if keywords.is_empty() {
        return;
    }

    let (my, next) = if let Some(my_next) = keywords.split_first() {
        my_next
    } else {
        return;
    };

    for my_kw in my {
        let mut job_kws = my_keywords.clone();
        job_kws.push(*my_kw);

        if next.is_empty() {
            let cloned_cb = callback.clone();
            scope.spawn(move |_| {
                cloned_cb(HashMap::from_iter(job_kws.into_iter()));
            });
        } else {
            compile_job_recursive(scope, job_kws, next, callback.clone());
        }
    }
}

/// Runs `callback` for each conbination of keywords as given by `keywords` in the [Scope] `scope`
fn start_compile_jobs<'a, 'scope>(
    scope: &Scope<'scope>,
    keywords: &HashMap<&'a str, KeywordValue>,
    callback: impl Fn(HashMap<&'a str, i64>) + Send + Sync + 'scope + Clone,
) where
    'a: 'scope,
{
    let jobs: Vec<Vec<(&'a str, i64)>> = keywords.iter().map(|(k, v)| v.flatten(*k)).collect();

    log::info!(
        "Running {} total compile jobs",
        jobs.iter().map(|kws| kws.len()).product::<usize>()
    );

    compile_job_recursive(scope, Vec::new(), &jobs, callback);
}

/// Compiles the given raw shader into its different variants, one per combination of keywords as given by `keywords`. The result
/// of each compilation job is passed to `callback` in order of completion (which is arbitrary)
///
/// Each [KeywordValue::Range] of length `N` in the `keywords` map increases the amount of jobs/variants multiplicatively. For example,
/// given two keywords with ranges of size `X` and `Y`, this spawns `X * Y` jobs. This function does not automatically throttle
/// the amount of concurrent jobs, and uses the existing [rayon] thread pool.
#[profiling::function]
pub fn compile_shader<'a>(
    raw_shader: &str,
    keywords: &HashMap<&'a str, KeywordValue>,
    stage: CompileStage,
    callback: impl Fn(Result<ShaderOutput<'a>, Error>) + Sync + Send,
) {
    log::info!("Compiling shader with {} keywords", keywords.len());

    let start = Instant::now();

    rayon::scope(|scope| {
        start_compile_jobs(scope, keywords, |job_keywords| {
            let result = compile_single_shader(raw_shader, job_keywords, stage);

            callback(result);
        })
    });

    log::info!(
        "Done after {} seconds",
        Instant::now().duration_since(start).as_secs_f32()
    );
}

/// Compiles a single shader variant and returns the corresponding output.
#[profiling::function]
pub fn compile_single_shader<'a>(
    raw_shader: &str,
    keywords: HashMap<&'a str, i64>,
    stage: CompileStage,
) -> Result<ShaderOutput<'a>, Error> {
    let mut shader = preprocessor::preprocess(raw_shader, &keywords)?;

    if stage <= CompileStage::Preprocess {
        return Ok(shader);
    }

    shader = compiler::compile_to_naga_ir(shader)?;

    Ok(shader)
}

/// An error during shader compilation
#[derive(Debug, Error)]
pub enum Error {
    /// Preprocessing failed
    #[error("Error during preprocessing: {0}")]
    Preprocess(#[from] PreprocessErr),

    #[error("Error parsing preprocessed WGSL shader: {0}")]
    Parse(#[from] naga::front::wgsl::ParseError),
}
