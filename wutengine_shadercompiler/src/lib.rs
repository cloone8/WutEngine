#![doc = include_str!("../README.md")]

use std::collections::{HashMap, HashSet};

use core::fmt::Write;
use nohash_hasher::IntSet;
use parser::{Condition, ParseErr, ShaderFile};
use smallvec::SmallVec;

mod parser;

/// Group index of the camera bind group
pub const CAMERA_PARAMS_BIND_GROUP_INDEX: u32 = 0;
/// Group index constant name of the camera bind group
pub const CAMERA_PARAMS_BIND_GROUP_KEYWORD: &str = "WUTENGINE_CAMERA_GROUP";

/// Group index of the material bind group
pub const MATERIAL_PARAMS_BIND_GROUP_INDEX: u32 = 1;

/// Group index constant name of the material bind group
pub const MATERIAL_PARAMS_BIND_GROUP_KEYWORD: &str = "WUTENGINE_MATERIAL_GROUP";

/// Group index of the per-instance bind group
pub const INSTANCE_PARAMS_BIND_GROUP_INDEX: u32 = 2;

/// Group index constant name of the per-instance bind group
pub const INSTANCE_PARAMS_BIND_GROUP_KEYWORD: &str = "WUTENGINE_INSTANCE_GROUP";

/// An implementation that provides deterministic hashes for a shader compilation
pub trait ShaderHasher<Id> {
    /// Converts the string ID of a shader to a hash value
    fn hash_source_id(id: Id) -> u64;

    /// Converts a map of keyword names and value to a single hash value
    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64;
}

/// Input data for a single [compile] job
#[derive(Debug, Clone)]
pub struct CompInput<'a, Id> {
    /// The ID of the source shader
    pub id: Id,

    /// The source shader content
    pub source: &'a str,

    /// The keywords to set
    pub keywords: &'a HashMap<String, u64>,

    /// The list of conditions for all the shader material parameters.
    /// The remaining parameter indices are returned in [CompOutput::remaining_params]
    pub parameters: &'a [Option<&'a str>],

    /// The list of conditions for all the shader vertex attributes.
    /// The remaining attribute indices are returned in [CompOutput::remaining_vertex_attributes]
    pub vertex_attributes: &'a [Option<&'a str>],

    /// The per-camera code block
    pub per_camera_block: &'a str,

    /// The per-instance code block
    pub per_instance_block: &'a str,
}

/// Output of a single succesful [compile] job
#[derive(Debug, Clone)]
pub struct CompOutput {
    /// The translated [naga] [module](naga::Module)
    pub module: Box<naga::Module>,

    /// The hashed source shader ID
    pub source_id_hash: u64,

    /// The hashed keyword value set
    pub keyword_hash: u64,

    /// Indices into [CompInput::parameters] of the parameters that have _not_ been stripped
    pub remaining_params: IntSet<usize>,

    /// Indices into [CompInput::vertex_attributes] of the attributes that have _not_ been stripped
    pub remaining_vertex_attributes: IntSet<usize>,
}

/// An error while compiling a shader with [compile]
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum CompileErr {
    /// Input was not valid, probably due to malformed directives
    #[display("Failed to parse input shader: {}", _0)]
    Parse(Box<ParseErr>),

    /// A directive that needs to be matched with another, wasn't
    #[display("Directive mismatch: {}", _0)]
    DirectiveMismatch(#[error(not(source))] &'static str),

    /// The WGSL after preprocessing was not valid
    #[display("Failed to compile preprocessed WGSL into a module: {}", _0)]
    CompileWgsl(naga::front::wgsl::ParseError),

    /// A keyword mentioned in a condition was not present
    #[display("Missing value for keyword \"{}\"", _0)]
    MissingKeywordValue(#[error(not(source))] String),
}

/// Compiles a single shader variant based on the input data provided by `input`.
///
/// Uses the hashing algorithm provided by `H`
pub fn compile<Id, H: ShaderHasher<Id>>(
    input: CompInput<'_, Id>,
) -> Result<CompOutput, CompileErr> {
    let source_id_hash = H::hash_source_id(input.id);
    let keyword_hash = H::hash_keywords(input.keywords);

    profiling::function_scope!(format!("{source_id_hash}#{keyword_hash}").as_str());

    log::info!("Compiling shader variant {source_id_hash}#{keyword_hash}");

    // First we parse the raw text into a set of source lines and compiler directives
    let parsed = ShaderFile::parse(input.source)?;

    // Then we apply the compiler directives, ending up with a new source file
    let mut applied = apply_branch_directives(parsed, input.keywords)?;

    // We prepend the per-camera and per-instance source blocks
    applied = format!(
        "{}\n{}\n{}",
        input.per_camera_block, input.per_instance_block, applied
    );

    // Replace all keyword references with their values
    inject_keywords_as_constants(&mut applied, input.keywords);

    // Find the set of remaining parameters based on the input parameter conditions
    log::debug!("Stripping shader parameters");
    let remaining_params = strip_by_conditions(input.parameters, input.keywords)?;

    log::debug!("Stripping shader vertex attributes");
    let remaining_vertex_attributes = strip_by_conditions(input.parameters, input.keywords)?;

    // Compile into a naga module
    let module = {
        profiling::scope!("Naga cross-compile");
        log::debug!("Compiling WGSL to Naga IR");

        let mut naga_frontend =
            naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
                parse_doc_comments: true,
            });

        Box::new(naga_frontend.parse(&applied)?)
    };

    log::debug!("Compiled shader variant {source_id_hash}#{keyword_hash}");

    Ok(CompOutput {
        module,
        source_id_hash,
        keyword_hash,
        remaining_params,
        remaining_vertex_attributes,
    })
}

/// Applies all branch directives contained in `file`, based on the conditions
/// evaluated from `keywords`
fn apply_branch_directives(
    file: ShaderFile,
    keywords: &HashMap<String, u64>,
) -> Result<String, CompileErr> {
    profiling::function_scope!();

    log::debug!("Applying branch directives");

    let mut out = String::new();

    let mut branch_stack: SmallVec<[bool; 32]> = SmallVec::new();

    for stmt in file.0 {
        match stmt {
            parser::Statement::Source(s) => {
                // Try to pop the state of the current #if/#else/#elif directive, if any.
                // If no directive is active, we're not in a branch so we can just write the source
                let active = branch_stack.last().copied().unwrap_or(true);

                if active {
                    writeln!(out, "{}", s).expect("Failed to write into string")
                }
            }
            parser::Statement::Directive(directive) => match directive {
                parser::Directive::If(condition) => {
                    let branch_active = condition
                        .eval(keywords)
                        .map_err(|e| CompileErr::MissingKeywordValue(e.to_owned()))?;

                    branch_stack.push(branch_active);
                }
                parser::Directive::Elif(condition) => {
                    let was_active = branch_stack
                        .pop()
                        .ok_or(CompileErr::DirectiveMismatch("\"elif\" without \"if\""))?;

                    if was_active {
                        branch_stack.push(false);
                    } else {
                        branch_stack.push(
                            condition
                                .eval(keywords)
                                .map_err(|e| CompileErr::MissingKeywordValue(e.to_owned()))?,
                        );
                    }
                }
                parser::Directive::Else => {
                    let was_active = branch_stack
                        .pop()
                        .ok_or(CompileErr::DirectiveMismatch("\"else\" without \"if\""))?;

                    branch_stack.push(!was_active);
                }
                parser::Directive::Endif => {
                    _ = branch_stack
                        .pop()
                        .ok_or(CompileErr::DirectiveMismatch("\"endif\" without \"if\""))?;
                }
            },
        }
    }

    if !branch_stack.is_empty() {
        return Err(CompileErr::DirectiveMismatch("\"if\" without \"endif\""));
    }

    Ok(out)
}

/// Evaluates a list of input conditions based on the provided keywords. Returns the set of indices into `condition_values`
/// that have _not_ been stripped.
fn strip_by_conditions(
    condition_values: &[Option<&str>],
    keywords: &HashMap<String, u64>,
) -> Result<IntSet<usize>, CompileErr> {
    profiling::function_scope!();

    let mut set = HashSet::default();

    for (i, &maybe_condition_string) in condition_values.iter().enumerate() {
        let Some(condition_string) = maybe_condition_string else {
            set.insert(i); // No condition means always active
            continue;
        };

        let condition = Condition::parse(condition_string)?;

        let active = condition
            .eval(keywords)
            .map_err(|e| CompileErr::MissingKeywordValue(e.to_owned()))?;

        if active {
            set.insert(i);
        }
    }

    log::debug!(
        "Stripped {} out of {} values",
        condition_values.len() - set.len(),
        condition_values.len()
    );

    Ok(set)
}

/// Looks for occurences of each keyword in the input string, and replaces them
/// with the concrete values given in `keywords`. Does not detect missing keywords
fn inject_keywords_as_constants(source: &mut String, keywords: &HashMap<String, u64>) {
    profiling::function_scope!();

    let mut replaced = 0;

    replaced += inject_keyword(
        source,
        CAMERA_PARAMS_BIND_GROUP_KEYWORD,
        CAMERA_PARAMS_BIND_GROUP_INDEX as u64,
    );

    replaced += inject_keyword(
        source,
        MATERIAL_PARAMS_BIND_GROUP_KEYWORD,
        MATERIAL_PARAMS_BIND_GROUP_INDEX as u64,
    );

    replaced += inject_keyword(
        source,
        INSTANCE_PARAMS_BIND_GROUP_KEYWORD,
        INSTANCE_PARAMS_BIND_GROUP_INDEX as u64,
    );

    for (keyword, &val) in keywords {
        replaced += inject_keyword(source, keyword, val);
    }

    log::debug!("Inserted {} total keyword values", replaced);
}

/// Injects a single keyword into the given source string, replacing
/// it with `val`. Returns the amount of replaced keywords
fn inject_keyword(source: &mut String, keyword: &str, val: u64) -> usize {
    let kw_byte_len = keyword.len();
    let val_string = val.to_string();

    let mut replaced = 0;

    while let Some(start) = source.find(keyword) {
        source.replace_range(start..(start + kw_byte_len), &val_string);
        replaced += 1;
    }

    replaced
}
