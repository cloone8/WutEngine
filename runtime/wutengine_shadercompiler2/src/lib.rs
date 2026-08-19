#![doc = include_str!("../README.md")]

use core::error::Error;
use std::collections::HashMap;

use wutengine_assets::assets::shader::PrecompiledShader;

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
pub struct Config {
    /// Enabled keyword values
    pub keywords: HashMap<String, u64>,

    /// The [`ShaderResolver`] to use
    pub shader_resolver: Option<Box<dyn ShaderResolver>>,
}

/// Compiles the given input using the provided config
pub fn compile(input: &str, config: &Config) -> Result<PrecompiledShader, CompileErr> {
    profiling::function_scope!();

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

    let module = {
        profiling::scope!("Naga parse");

        let mut naga_frontend =
            naga::front::wgsl::Frontend::new_with_options(naga::front::wgsl::Options {
                parse_doc_comments: true,
                capabilities: naga::valid::Capabilities::default(),
            });

        naga_frontend
            .parse(&preprocessed_source)
            .map_err(Box::new)
            .map(Box::new)?
    };

    let parameters = parameters::find_parameters(&module);

    Ok(PrecompiledShader { module, parameters })
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
