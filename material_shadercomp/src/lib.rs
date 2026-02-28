use std::collections::{HashMap, HashSet};

use core::fmt::Write;
use parser::{ParseErr, ShaderFile, parse_condition};

mod parser;

pub const CAMERA_PARAMS_BIND_GROUP_INDEX: u32 = 0;
pub const CAMERA_PARAMS_BIND_GROUP_KEYWORD: &str = "WUTENGINE_CAMERA_GROUP";

pub const MATERIAL_PARAMS_BIND_GROUP_INDEX: u32 = 1;
pub const MATERIAL_PARAMS_BIND_GROUP_KEYWORD: &str = "WUTENGINE_MATERIAL_GROUP";

pub const INSTANCE_PARAMS_BIND_GROUP_INDEX: u32 = 2;
pub const INSTANCE_PARAMS_BIND_GROUP_KEYWORD: &str = "WUTENGINE_INSTANCE_GROUP";

/// An implementation that provides deterministic hashes for a shader compilation
pub trait ShaderHasher<Id> {
    /// Converts the string ID of a shader to a hash value
    fn hash_source_id(id: Id) -> u64;

    /// Converts a map of keyword names and value to a single hash value
    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64;
}

#[derive(Debug)]
pub struct CompInput<'a, Id> {
    pub id: Id,
    pub source: &'a str,
    pub keywords: &'a HashMap<String, u64>,
    pub user_params: &'a [Option<&'a str>],
    pub vertex_attributes: &'a [Option<&'a str>],
    pub per_camera_block: &'a str,
    pub per_instance_block: &'a str,
}

#[derive(Debug)]
pub struct CompOutput {
    pub module: Box<naga::Module>,
    pub source_id_hash: u64,
    pub keyword_hash: u64,
    pub remaining_params: HashSet<usize>,
    pub remaining_vertex_attributes: HashSet<usize>,
}

#[derive(Debug, derive_more::From, derive_more::Display, derive_more::Error)]
pub enum CompileErr {
    #[display("Failed to parse input shader: {}", _0)]
    Parse(Box<ParseErr>),

    #[display("Directive mismatch: {}", _0)]
    DirectiveMismatch(#[error(not(source))] &'static str),

    #[display("Failed to compile preprocessed WGSL into a module: {}", _0)]
    CompileWgsl(naga::front::wgsl::ParseError),

    #[display("Missing value for keyword \"{}\"", _0)]
    MissingKeywordValue(#[error(not(source))] String),
}

pub fn compile<Id, H: ShaderHasher<Id>>(
    input: CompInput<'_, Id>,
) -> Result<CompOutput, CompileErr> {
    // First we parse the raw text into a set of source lines and compiler directives
    let parsed = ShaderFile::parse(input.source)?;

    // Then we apply the compiler directives, ending up with a new source file
    let mut applied = apply_directives(parsed, input.keywords)?;

    // We prepend the per-camera and per-instance source blocks
    applied = format!(
        "{}\n{}\n{}",
        input.per_camera_block, input.per_instance_block, applied
    );

    // Replace all keyword references with their values
    inject_keywords_as_constants(&mut applied, input.keywords);

    // Find the set of remaining parameters based on the input parameter conditions
    let remaining_params = strip_parameters(input.user_params, input.keywords)?;
    let remaining_vertex_attributes = strip_parameters(input.user_params, input.keywords)?;

    // Compile into a naga module
    let mut naga_frontend =
        naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
            parse_doc_comments: true,
        });

    let module = Box::new(naga_frontend.parse(&applied)?);

    Ok(CompOutput {
        module,
        source_id_hash: H::hash_source_id(input.id),
        keyword_hash: H::hash_keywords(input.keywords),
        remaining_params,
        remaining_vertex_attributes,
    })
}

fn apply_directives(
    file: ShaderFile,
    keywords: &HashMap<String, u64>,
) -> Result<String, CompileErr> {
    let mut out = String::new();

    let mut branch_stack: Vec<bool> = Vec::new();

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

fn strip_parameters(
    user_param_conditions: &[Option<&str>],
    keywords: &HashMap<String, u64>,
) -> Result<HashSet<usize>, CompileErr> {
    let mut set = HashSet::default();

    for (i, &maybe_condition_string) in user_param_conditions.iter().enumerate() {
        let Some(condition_string) = maybe_condition_string else {
            set.insert(i); // No condition means always active
            continue;
        };

        let condition = parse_condition(condition_string)?;

        let active = condition
            .eval(keywords)
            .map_err(|e| CompileErr::MissingKeywordValue(e.to_owned()))?;

        if active {
            set.insert(i);
        }
    }

    Ok(set)
}

fn inject_keywords_as_constants(source: &mut String, keywords: &HashMap<String, u64>) {
    inject_keyword(
        source,
        CAMERA_PARAMS_BIND_GROUP_KEYWORD,
        CAMERA_PARAMS_BIND_GROUP_INDEX as u64,
    );

    inject_keyword(
        source,
        MATERIAL_PARAMS_BIND_GROUP_KEYWORD,
        MATERIAL_PARAMS_BIND_GROUP_INDEX as u64,
    );

    inject_keyword(
        source,
        INSTANCE_PARAMS_BIND_GROUP_KEYWORD,
        INSTANCE_PARAMS_BIND_GROUP_INDEX as u64,
    );

    for (keyword, &val) in keywords {
        inject_keyword(source, keyword, val);
    }
}

fn inject_keyword(source: &mut String, keyword: &str, val: u64) {
    let kw_byte_len = keyword.len();
    let val_string = val.to_string();

    while let Some(start) = source.find(keyword) {
        source.replace_range(start..(start + kw_byte_len), &val_string);
    }
}
