//! Cross compilation to OpenGL 4.1

use std::collections::HashMap;

use bitflags::Flags;
use naga::WithSpan;
use naga::back::glsl::{self};
use naga::front::wgsl::ParseError;
use naga::proc::BoundsCheckPolicies;
use naga::valid::{Capabilities, SubgroupOperationSet, ValidationError, ValidationFlags};
use thiserror::Error;
use wutengine_graphics::shader::builtins::ShaderBuiltins;
use wutengine_graphics::shader::uniform::{SingleUniformBinding, Uniform, UniformBinding};
use wutengine_graphics::shader::{GLShaderMeta, RawShader, ShaderStage};

#[derive(Debug, Error)]
pub enum CrossOpenGLErr {
    #[error("Input parsing error: {0}")]
    Parse(#[from] ParseError),

    #[error("Input was invalid: {0}")]
    Validation(#[from] WithSpan<ValidationError>),

    #[error("Error compiling the validated input to GLSL: {0}")]
    Compile(#[from] glsl::Error),

    #[error("Uniform {0} was remapped twice. First: {1:?}, second {2:?}")]
    DoubleUniform(String, Box<Uniform>, Box<Uniform>),

    #[error("Uniform {0} with binding {1:?} was not remapped")]
    MissingUniform(String, Box<Uniform>),
}

#[derive(Debug, Clone)]
#[repr(transparent)]
struct BindingMap(Vec<(usize, usize)>);

impl BindingMap {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn get_or_insert(&mut self, group: usize, binding: usize) -> usize {
        let as_tuple = (group, binding);

        match self.0.iter().position(|elem| elem == &as_tuple) {
            Some(pos) => pos,
            None => {
                self.0.push(as_tuple);
                self.0.len() - 1
            }
        }
    }

    fn update_binding(&mut self, binding: &mut SingleUniformBinding) {
        let cur_group = binding.group;
        let cur_binding = binding.binding;

        let new_binding = self.get_or_insert(cur_group, cur_binding);

        binding.group = 0;
        binding.binding = new_binding;
    }
}

pub(super) fn cross_to_opengl(shader: &mut RawShader) -> Result<GLShaderMeta, CrossOpenGLErr> {
    log::info!("Starting OpenGL cross compile for shader {}", shader.ident);

    let mut metadata = GLShaderMeta {
        builtins_vertex: HashMap::new(),
        builtins_fragment: HashMap::new(),
    };

    let mut binding_map = BindingMap::new();

    let mut remapped_uniforms = HashMap::with_capacity(shader.uniforms.len());

    if let Some(vtx) = &mut shader.source.vertex {
        log::debug!("Cross compiling vertex shader");
        cross_stage(
            vtx,
            naga::ShaderStage::Vertex,
            shader.builtins,
            &shader.uniforms,
            &mut binding_map,
            &mut remapped_uniforms,
            &mut metadata.builtins_vertex,
        )?;
    } else {
        log::debug!("No vertex shader to cross compile");
    }

    if let Some(frag) = &mut shader.source.fragment {
        log::debug!("Cross compiling fragment shader");
        cross_stage(
            frag,
            naga::ShaderStage::Fragment,
            shader.builtins,
            &shader.uniforms,
            &mut binding_map,
            &mut remapped_uniforms,
            &mut metadata.builtins_fragment,
        )?;
    } else {
        log::debug!("No fragment shader to cross compile");
    }

    shader.uniforms = remapped_uniforms;

    Ok(metadata)
}

fn cross_stage(
    stage_src: &mut ShaderStage,
    stage: naga::ShaderStage,
    builtins: ShaderBuiltins,
    uniforms: &HashMap<String, Uniform>,
    binding_map: &mut BindingMap,
    remapped_uniforms: &mut HashMap<String, Uniform>,
    remapped_builtins: &mut HashMap<ShaderBuiltins, SingleUniformBinding>,
) -> Result<(), CrossOpenGLErr> {
    log::trace!("Parsing stage");

    let parsed = naga::front::wgsl::parse_str(&stage_src.source)?;

    log::trace!("Validating stage");

    let info = naga::valid::Validator::new(ValidationFlags::default(), Capabilities::default())
        .subgroup_stages(to_valid_shaderstages(stage))
        .subgroup_operations(SubgroupOperationSet::all())
        .validate(&parsed)?;

    let options = glsl::Options {
        version: glsl::Version::Desktop(410),
        writer_flags: glsl::WriterFlags::empty(),
        ..Default::default()
    };

    let pipeline_options = glsl::PipelineOptions {
        shader_stage: stage,
        entry_point: stage_src.entry.clone(),
        multiview: None,
    };

    log::trace!("Writing output");
    let mut out = String::new();

    let mut writer = glsl::Writer::new(
        &mut out,
        &parsed,
        &info,
        &options,
        &pipeline_options,
        BoundsCheckPolicies::default(),
    )?;

    let reflection_info = writer.write()?;

    stage_src.source = out;
    stage_src.entry = "main".to_string();

    log::trace!("Remapping builtins");
    for builtin in builtins.iter().filter(|bi| !bi.contains_unknown_bits()) {
        let binding = builtin.binding();
        let handle = get_globvar_handle(&parsed.global_variables, &binding.name);

        if let Some(handle) = handle {
            let mut new_binding = binding.clone();
            new_binding.name = reflection_info.uniforms[&handle].clone();
            binding_map.update_binding(&mut new_binding);
            remapped_builtins.insert(builtin, new_binding);
        }
    }

    for (uform_name, uform_val) in uniforms {
        log::trace!("Remapping uniform {}", uform_name);

        if uform_val.ty.is_texture_type() {
            let (tex_binding, samp_binding) = match uform_val.binding.try_as_texture() {
                Some(b) => b,
                None => {
                    log::error!(
                        "Non-texture binding descriptor for texture {}, skipping",
                        uform_name
                    );
                    continue;
                }
            };

            let tex_binding_name = &tex_binding.unwrap().name;
            let samp_binding_name = &samp_binding.unwrap().name;

            let tex_handle = get_globvar_handle(&parsed.global_variables, tex_binding_name);
            let sampler_handle = get_globvar_handle(&parsed.global_variables, samp_binding_name);

            assert_eq!(tex_handle.is_some(), sampler_handle.is_some());

            if let (Some(tex), Some(samp)) = (tex_handle, sampler_handle) {
                for (new_sampler_name, tex_mapping) in &reflection_info.texture_mapping {
                    if tex_mapping.texture == tex {
                        assert_eq!(Some(samp), tex_mapping.sampler);

                        log::trace!(
                            "New binding name for texture {} is {}",
                            uform_name,
                            new_sampler_name
                        );

                        let mut new_sampler_binding = samp_binding.unwrap().clone();
                        new_sampler_binding.name = new_sampler_name.clone();
                        binding_map.update_binding(&mut new_sampler_binding);

                        let mut new_uform_val = uform_val.clone();
                        new_uform_val.binding = UniformBinding::Texture {
                            sampler: Some(new_sampler_binding),
                            texture: None,
                        };

                        if remapped_uniforms.contains_key(uform_name) {
                            return Err(CrossOpenGLErr::DoubleUniform(
                                uform_name.clone(),
                                Box::new(remapped_uniforms[uform_name].clone()),
                                Box::new(new_uform_val),
                            ));
                        }

                        remapped_uniforms.insert(uform_name.clone(), new_uform_val);

                        break;
                    }
                }
            }
        } else {
            let input_binding = match uform_val.binding.try_as_standard() {
                Some(b) => b,
                None => {
                    log::error!(
                        "Texture binding descriptor for non-texture {}, skipping",
                        uform_name
                    );
                    continue;
                }
            };

            let uniform_handle = get_globvar_handle(&parsed.global_variables, &input_binding.name);

            if uniform_handle.is_none() {
                continue;
            }

            let uniform_handle = uniform_handle.unwrap();

            let new_binding_name = reflection_info.uniforms[&uniform_handle].clone();

            let mut new_binding = input_binding.clone();
            new_binding.name = new_binding_name;
            binding_map.update_binding(&mut new_binding);

            log::trace!("New binding for {} is {:#?}", uform_name, new_binding);

            let mut new_uform_val = uform_val.clone();
            new_uform_val.binding = UniformBinding::Standard(new_binding);

            if remapped_uniforms.contains_key(uform_name) {
                return Err(CrossOpenGLErr::DoubleUniform(
                    uform_name.clone(),
                    Box::new(remapped_uniforms[uform_name].clone()),
                    Box::new(new_uform_val),
                ));
            }

            remapped_uniforms.insert(uform_name.clone(), new_uform_val);
        }
    }

    Ok(())
}

fn to_valid_shaderstages(s: naga::ShaderStage) -> naga::valid::ShaderStages {
    match s {
        naga::ShaderStage::Vertex => naga::valid::ShaderStages::VERTEX,
        naga::ShaderStage::Fragment => naga::valid::ShaderStages::FRAGMENT,
        naga::ShaderStage::Compute => naga::valid::ShaderStages::COMPUTE,
        naga::ShaderStage::Task | naga::ShaderStage::Mesh => {
            panic!("Unsupported shader stage: {:?}", s)
        }
    }
}
fn get_globvar_handle(
    globvars: &naga::Arena<naga::GlobalVariable>,
    name: impl AsRef<str>,
) -> Option<naga::Handle<naga::GlobalVariable>> {
    globvars.fetch_if(|global| global.name.as_ref().unwrap() == name.as_ref())
}
