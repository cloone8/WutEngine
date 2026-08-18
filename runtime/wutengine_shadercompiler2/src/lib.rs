#![doc = include_str!("../README.md")]

use core::error::Error;
use std::collections::HashMap;

use crate::preprocessor::PreprocessErr;

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

    /// Failed to parse parameters
    #[display("Failed to parse parameters in shader: {_0}")]
    Parameters(FindParametersErr),
}

/// Output of a compile job
#[derive(Debug)]
pub struct CompileOutput {
    /// The compiled module
    pub compiled_module: naga::Module,

    /// The parameters the shader has
    pub parameters: HashMap<ParameterBinding, Parameter>,
}

/// The binding for a [`Parameter`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParameterBinding {
    /// Buffer binding. A parameter within a buffer
    Buffer {
        /// The group
        group: u32,

        /// The binding
        binding: u32,
    },

    /// Opaque binding. A parameter that should be bound directly
    Opaque {
        /// The group
        group: u32,
        /// The binding
        binding: u32,
    },
}

/// Information on an exposed parameter in a shader
#[derive(Debug, Clone)]
pub struct Parameter {}

/// A compilation job configuration
#[derive(Debug)]
pub struct Config {
    /// Enabled keyword values
    pub keywords: HashMap<String, u64>,

    /// The [`ShaderResolver`] to use
    pub shader_resolver: Option<Box<dyn ShaderResolver>>,
}

/// Compiles the given input using the provided config
pub fn compile(input: &str, config: &Config) -> Result<Box<CompileOutput>, CompileErr> {
    log::info!("Compiling input");

    let preprocessed_source = preprocessor::preprocess(
        input,
        config.keywords.clone(),
        config
            .shader_resolver
            .as_deref()
            .unwrap_or(&UnsupportedResolver),
    )?;

    log::info!("Preprocessing result:\n{preprocessed_source}");

    log::debug!("Parsing WGSL to Naga IR");

    let mut naga_frontend =
        naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
            parse_doc_comments: true,
            capabilities: naga::valid::Capabilities::default(),
        });

    let module = naga_frontend
        .parse(&preprocessed_source)
        .map_err(Box::new)?;

    let parameters = find_parameters(&module)?;

    Ok(Box::new(CompileOutput {
        compiled_module: module,
        parameters,
    }))
}

/// An error while resolving parameters with [`find_parameters`]
#[derive(Debug, derive_more::Error, derive_more::Display)]
pub enum FindParametersErr {}

/// Find the exposed parameters in a [`naga::Module`]
pub fn find_parameters(
    module: &naga::Module,
) -> Result<HashMap<ParameterBinding, Parameter>, FindParametersErr> {
    log::debug!("Finding parameters for module");

    let mut params = HashMap::new();

    for (_, global) in module.global_variables.iter() {
        log::info!("{:?}", global.name);
    }

    Ok(params)
}

/// A type that can resolve shader source by name
pub trait ShaderResolver: core::fmt::Debug + Send + Sync {
    /// For a given shader name, returns the source code.
    fn find_by_name(&self, shader_name: &str) -> Result<String, Box<dyn Error>>;
}

/// Simple internal shader resolver that always errors
#[derive(Debug)]
struct UnsupportedResolver;

impl ShaderResolver for UnsupportedResolver {
    fn find_by_name(&self, shader_name: &str) -> Result<String, Box<dyn Error>> {
        #[derive(Debug, derive_more::Error, derive_more::Display)]
        #[display("No resolver was given")]
        struct Unsupported;

        _ = shader_name;

        Err(Box::new(Unsupported))
    }
}
