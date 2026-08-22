//! Vertex shader input resolution

use core::str::FromStr;

use naga::Binding;
use naga::Handle;
use naga::Module;
use naga::ShaderStage;
use naga::Type;
use naga::TypeInner;
use wutengine_assets::assets::shader::ShaderVertexAttributeType;
use wutengine_assets::assets::shader::VertexInput;
use wutengine_assets::nohash_hasher::IntMap;
use wutengine_assets::nohash_hasher::IntSet;

/// An error while trying to find vertex inputs with [`find_vertex_inputs`]
#[derive(Debug, derive_more::Display, derive_more::From, derive_more::Error)]
pub enum FindVertexInputsErr {
    /// Could not find an entrypoint
    #[display("Could not find vertex shader entry point")]
    MissingEntrypoint,

    /// Too many entrypoints found
    #[display("Encountered more than one vertex entrypoint. First: {first}, next: {next}")]
    TooManyEntrypoints {
        /// First entrypoint
        first: String,
        /// Next entrypoint
        next: String,
    },

    /// Unknown attribute
    #[display("Unknown attribute: {_0}")]
    UnknownAttribute(#[error(not(source))] String),

    /// Duplicate attribute
    #[display("Duplicate attribute found: {_0}")]
    DuplicateAttribute(#[error(not(source))] ShaderVertexAttributeType),
}

/// Find all vertex inputs for the given module
pub fn find_vertex_inputs(
    module: &Module,
) -> Result<IntMap<u32, VertexInput>, FindVertexInputsErr> {
    profiling::function_scope!();

    let mut vertex_entry: Option<&naga::EntryPoint> = None;

    for entry_point in &module.entry_points {
        if entry_point.stage == ShaderStage::Vertex {
            match vertex_entry {
                Some(prev_found_entry_point) => {
                    return Err(FindVertexInputsErr::TooManyEntrypoints {
                        first: prev_found_entry_point.name.clone(),
                        next: entry_point.name.clone(),
                    });
                }
                None => {
                    vertex_entry = Some(entry_point);
                }
            }
        }
    }

    let Some(vertex_entry) = vertex_entry else {
        return Err(FindVertexInputsErr::MissingEntrypoint);
    };

    let mut vertex_inputs = IntMap::default();

    for argument in &vertex_entry.function.arguments {
        parse_vertex_input(
            argument.name.as_deref(),
            argument.binding.as_ref(),
            argument.ty,
            module,
            &mut vertex_inputs,
        )?;
    }

    // Final check for duplicate attributes
    let mut found_attributes: IntSet<u16> = IntSet::default();

    for input in vertex_inputs.values() {
        let attr_u16 = input.attribute.as_u16();

        if !found_attributes.insert(attr_u16) {
            return Err(FindVertexInputsErr::DuplicateAttribute(input.attribute));
        }
    }

    Ok(vertex_inputs)
}

fn parse_vertex_input_name(name: &str) -> Result<ShaderVertexAttributeType, FindVertexInputsErr> {
    match name.trim().to_lowercase().as_str() {
        "position" | "pos" => Ok(ShaderVertexAttributeType::Position),
        "normal" => Ok(ShaderVertexAttributeType::Normal),
        "color" => Ok(ShaderVertexAttributeType::Color),
        other => {
            let normalized = other.trim().to_lowercase();

            if normalized.starts_with("uv")
                || normalized.starts_with("texcoord")
                || normalized.starts_with("tex_coord")
            {
                let channel = parse_uv_channel(&normalized)
                    .ok_or_else(|| FindVertexInputsErr::UnknownAttribute(other.to_string()))?;

                return Ok(ShaderVertexAttributeType::Uv { channel });
            }

            Err(FindVertexInputsErr::UnknownAttribute(other.to_string()))
        }
    }
}

fn parse_uv_channel(name: &str) -> Option<u8> {
    let numbers = name
        .split(|c: char| !c.is_ascii_digit())
        .filter(|substr| !substr.trim().is_empty())
        .collect::<Vec<_>>();

    if numbers.is_empty() {
        // Assume channel 0
        return Some(0);
    }

    if numbers.len() > 1 {
        // We cannot parse multiple numbers
        return None;
    }

    let number = numbers.into_iter().next().unwrap();

    u8::from_str(number).ok()
}

fn parse_vertex_input(
    name: Option<&str>,
    binding: Option<&Binding>,
    arg_type: Handle<Type>,
    module: &Module,
    output: &mut IntMap<u32, VertexInput>,
) -> Result<(), FindVertexInputsErr> {
    if let Some(binding) = binding {
        // Binding in argument itself

        let location = match binding {
            naga::Binding::Location { location, .. } => *location,
            naga::Binding::BuiltIn(built_in) => {
                log::debug!("Skipping built-in vertex input: {name:?}/{built_in:?}");
                return Ok(());
            }
        };

        let attribute = parse_vertex_input_name(name.as_ref().expect("Missing name?"))?;

        let prev = output.insert(location, VertexInput { attribute });

        assert!(
            prev.is_none(),
            "Duplicate vertex location in validated module"
        );
    } else {
        // Binding probably in referenced type
        let arg_type = &module.types[arg_type];

        parse_vertex_input_from_type(&arg_type.inner, module, output)?;
    }

    Ok(())
}

fn parse_vertex_input_from_type(
    arg_type: &TypeInner,
    module: &Module,
    output: &mut IntMap<u32, VertexInput>,
) -> Result<(), FindVertexInputsErr> {
    let TypeInner::Struct { members, .. } = arg_type else {
        unreachable!("Non-struct cannot be a vertex input");
    };

    for member in members {
        parse_vertex_input(
            member.name.as_deref(),
            member.binding.as_ref(),
            member.ty,
            module,
            output,
        )?;
    }

    Ok(())
}
