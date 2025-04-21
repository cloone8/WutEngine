//! Uniform functionality for OpenGL shaders

use core::ffi::CStr;
use core::num::NonZero;
use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::process::exit;

use glam::Mat4;
use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::renderer::RendererTextureId;
use wutengine_graphics::shader::{SingleUniformBinding, Uniform, UniformBinding, UniformType};

use crate::error::checkerr;
use crate::gltypes::{GlMat4f, size_of_gl};
use crate::opengl::types::{GLenum, GLint, GLintptr, GLsizeiptr, GLuint};
use crate::opengl::{self, Gl};
use crate::shader::reflection::{
    get_active_uniform, get_active_uniform_block_indices, get_active_uniform_block_uniforms,
    get_active_uniforms_iv, get_uniform_block_index, get_uniform_location,
};
use crate::texture::GlTexture;

use super::{GlShaderUniform, UniformBlockDescriptor};

/// Finds all uniforms in the given program, and returns information on them
pub(crate) fn discover_uniforms(
    gl: &Gl,
    program: NonZero<GLuint>,
    source: &HashMap<String, Uniform>,
) -> HashMap<String, GlShaderUniform> {
    log::debug!("Discovering uniforms for shaderprogram {}", program);

    let mut output = HashMap::with_capacity(source.len());

    for (uform_name, uform_desc) in source {
        log::info!("Discovering uniform {}", uform_name);
        if let Some(discovered) = discover_uniform(gl, program, uform_name, uform_desc) {
            output.insert(uform_name.clone(), discovered);
        } else {
            log::error!(
                "Could not find binding for uniform {}. Will not be assignable",
                uform_name
            )
        }
    }

    log::info!("Discovered uniforms: {:#?}", output);

    output
}

fn discover_uniform(
    gl: &Gl,
    program: NonZero<GLuint>,
    name: &String,
    desc: &Uniform,
) -> Option<GlShaderUniform> {
    if desc.ty.is_texture_type() {
        discover_texture(gl, program, name, desc)
    } else {
        discover_block(gl, program, name, desc)
    }
}

fn discover_texture(
    gl: &Gl,
    program: NonZero<GLuint>,
    name: &String,
    desc: &Uniform,
) -> Option<GlShaderUniform> {
    assert!(desc.ty.is_texture_type());

    let binding = match &desc.binding {
        UniformBinding::Standard(b) => {
            log::error!(
                "Texture type uniform {} has non-texture binding {}. Cannot resolve",
                name,
                b
            );
            return None;
        }
        UniformBinding::Texture {
            sampler,
            texture: _,
        } => match sampler {
            Some(b) => b,
            None => {
                log::error!(
                    "Texture uniform {} is missing sampler binding. Cannot resolve",
                    name
                );
                return None;
            }
        },
    };

    let loc = get_uniform_location(gl, program, &binding.name);

    if loc.is_none() {
        log::error!(
            "Could not find location for texture uniform {} with binding {}",
            name,
            binding
        );

        return None;
    }

    let location = loc.unwrap();

    Some(GlShaderUniform::Sampler { location })
}

fn discover_block(
    gl: &Gl,
    program: NonZero<GLuint>,
    name: &String,
    desc: &Uniform,
) -> Option<GlShaderUniform> {
    assert!(!desc.ty.is_texture_type());

    let binding = match desc.binding.try_as_standard() {
        Some(b) => b,
        None => {
            log::error!(
                "Standard type uniform {} has texture binding {}. Cannot resolve",
                name,
                desc.binding
            );
            return None;
        }
    };

    let index = match get_uniform_block_index(gl, program, &binding.name) {
        Some(i) => i,
        None => {
            log::error!(
                "Could not find index for uniform block {} with binding {}",
                name,
                binding
            );
            return None;
        }
    };

    let desc = match build_uniform_block_descriptor(gl, program, index) {
        Some(d) => d,
        None => {
            log::error!(
                "Could not build uniform block descriptor for block {} with binding {} and index {}",
                name,
                binding,
                index
            );
            return None;
        }
    };

    Some(GlShaderUniform::Block { index, desc })
}

fn build_uniform_block_descriptor(
    gl: &Gl,
    program: NonZero<GLuint>,
    index: GLuint,
) -> Option<UniformBlockDescriptor> {
    let block_uniforms = get_active_uniform_block_uniforms(gl, program, index);

    assert_ne!(0, block_uniforms);

    let mut indices = vec![opengl::INVALID_INDEX; block_uniforms];
    unsafe { get_active_uniform_block_indices(gl, program, index, &mut indices) };

    assert!(!indices.iter().copied().any(|i| i == opengl::INVALID_INDEX));

    let mut names = Vec::with_capacity(indices.len());
    let mut array_sizes = Vec::with_capacity(indices.len());
    let mut types = Vec::with_capacity(indices.len());

    indices
        .iter()
        .copied()
        .map(|i| get_active_uniform(gl, program, i))
        .for_each(|info| {
            names.push(info.name);
            array_sizes.push(info.size);
            types.push(info.ty);
        });

    log::info!("{:#?}", names);
    let offsets = get_active_uniforms_iv(gl, program, &indices, opengl::UNIFORM_OFFSET);
    assert_eq!(indices.len(), offsets.len());

    let array_strides = get_active_uniforms_iv(gl, program, &indices, opengl::UNIFORM_ARRAY_STRIDE);
    assert_eq!(indices.len(), array_strides.len());

    let mut members = Vec::with_capacity(block_uniforms);

    for i in 0..indices.len() {
        members.push(UniformBlockMemberInfo {
            path: get_path_from_uniform_block_member_name(&names[i]),
            array_size: array_sizes[i],
            ty: types[i],
            offset: offsets[i],
            stride: array_strides[i],
        });
    }

    assert_ne!(0, members.len());

    let mut members_mapped = create_member_map(&members);

    log::info!("{:#?}", members_mapped);

    flatten_member_arrays(&mut members_mapped);

    log::info!("{:#?}", members_mapped);
    // let ubd = build_subdescriptor(members, true);

    // Some(ubd)
    todo!()
}

#[derive(Debug, Clone)]
enum MemberInfoSet<'a> {
    Single(UniformBlockMemberInfo<'a>),
    Array {
        count: usize,
        stride: usize,
        info: UniformBlockMemberInfo<'a>,
    },
    Map(HashMap<String, MemberInfoSet<'a>>),
}

fn create_member_map<'a>(
    members: &[UniformBlockMemberInfo<'a>],
) -> HashMap<String, MemberInfoSet<'a>> {
    let mut map = HashMap::new();

    for member in members {
        let mut cur_root = &mut map;

        for path_segment in &member.path[..(member.path.len() - 1)] {
            if !cur_root.contains_key(*path_segment) {
                cur_root.insert(path_segment.to_string(), MemberInfoSet::Map(HashMap::new()));
            }

            cur_root = match cur_root.get_mut(*path_segment).unwrap() {
                MemberInfoSet::Single(_) => unreachable!(),
                MemberInfoSet::Array { .. } => unreachable!(),
                MemberInfoSet::Map(hash_map) => hash_map,
            }
        }

        cur_root.insert(
            member.path[member.path.len() - 1].to_string(),
            MemberInfoSet::Single(member.clone()),
        );
    }

    map
}

fn flatten_member_arrays<'a>(members: &mut HashMap<String, MemberInfoSet<'a>>) {
    for member in members.values_mut() {
        if let MemberInfoSet::Map(hm) = member {
            flatten_member_arrays(hm);
        }
    }

    let mut flattened = HashMap::<String, MemberInfoSet<'a>>::new();

    for (name, member) in members.iter() {
        if !name.contains('[') {
            flattened.insert(name.clone(), member.clone());
            continue;
        }

        let no_array = name.split_once('[').unwrap().0;

        if flattened.contains_key(no_array) {
            continue;
        }

        let mut matching_array_members = members
            .iter()
            .filter(|m| m.0.starts_with(no_array))
            .map(|m| m.1)
            .cloned()
            .collect::<Vec<_>>();

        matching_array_members.sort_by_key(|m| lowest_offset(m));

        let base = matching_array_members[0];

        todo!()
    }
}

fn lowest_offset<'a>(member: &'a MemberInfoSet<'a>) -> usize {
    match member {
        MemberInfoSet::Single(uniform_block_member_info) => {
            uniform_block_member_info.offset as usize
        }
        MemberInfoSet::Array {
            count: _,
            stride: _,
            info,
        } => info.offset as usize,
        MemberInfoSet::Map(hash_map) => hash_map.values().map(|v| lowest_offset(v)).min().unwrap(),
    }
}

fn get_path_from_uniform_block_member_name(name: &str) -> Vec<&str> {
    name.split(".").collect::<Vec<_>>()
}

#[derive(Debug, Clone)]
struct UniformBlockMemberInfo<'a> {
    path: Vec<&'a str>,
    array_size: usize,
    ty: GLenum,
    offset: GLint,
    stride: GLint,
}
