use core::num::NonZero;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::graphics::cache;
use crate::graphics::shader::compile;

use super::BindGroup;
use super::shader::{CompiledShader, Shader};

#[derive(Debug)]
pub(crate) struct Material {
    pub(crate) shader: Shader,
    pub(crate) keywords: HashMap<String, u64>,
    pub(crate) compiled: Arc<CompiledShader>,
    pub(crate) user_bind_group: BindGroup,
}

impl Material {
    pub(crate) fn new(shader: Shader, keywords: HashMap<String, u64>) -> Self {
        let compiled_shader = compile(&shader, &keywords).expect("Failed to compile shader");

        Self {
            shader,
            keywords,
            user_bind_group: BindGroup::new(
                "Material User Bind Group".to_string(),
                compiled_shader.user_bind_group_layout.clone(),
                &compiled_shader.parameters,
            ),
            compiled: compiled_shader,
        }
        // let variant_id = crate::graphics::shader::calculate_variant_id(shader.id, &keywords);

        // if let Some(cached) = cache::shader::find(&variant_id) {
        //     return Self {
        //         shader,
        //         keywords,
        //         compiled: cached,
        //         user_bind_group: BindGroup::new(
        //             "Material User Bind Group".to_string(),
        //             cached.user_bind_group_layout,
        //             &cached.parameters,
        //         ),
        //     };
        // }

        // let vertex_attr_conditions: Vec<Option<&str>> = Vec::from_iter(
        //     shader
        //         .vertex_attributes
        //         .iter()
        //         .map(|p| p.condition.as_ref().map(|c| c.0.as_str())),
        // );

        // let user_param_conditions: Vec<Option<&str>> = Vec::from_iter(
        //     shader
        //         .user_params
        //         .iter()
        //         .map(|p| p.get_condition().map(|c| c.0.as_str())),
        // );

        // let output = wutengine_shadercompiler::compile::<_, FakeHasher>(CompInput {
        //     id: shader.id as u64,
        //     source: shader.get_source(),
        //     keywords: &keywords,
        //     vertex_attributes: &vertex_attr_conditions,
        //     user_params: &user_param_conditions,
        //     per_camera_block: include_str!("camera.wgsl"),
        //     per_instance_block: include_str!("instance.wgsl"),
        // })
        // .unwrap();

        // let user_bind_group_layout: wgpu::BindGroupLayout = create_user_params_bind_group_layout(
        //     "Material BGL",
        //     &shader.user_params,
        //     &output.remaining_params,
        //     device,
        // );

        // let vertex_attributes = output
        //     .remaining_vertex_attributes
        //     .into_iter()
        //     .map(|idx| {
        //         let attr = &shader.vertex_attributes[idx];
        //         (attr.ty, attr.location)
        //     })
        //     .collect();

        // let compiled = CompiledShader {
        //     module: output.module,
        //     source_id_hash: output.source_id_hash,
        //     keyword_hash: output.keyword_hash,
        //     user_bind_group_layout: user_bind_group_layout.clone(),
        //     vertex_attributes,
        // };

        // Self {
        //     keywords,
        //     compiled,
        //     user_bind_group: BindGroup::new(
        //         "Material User Bind Group".to_string(),
        //         user_bind_group_layout,
        //         shader.user_params.iter().enumerate().filter_map(|(i, p)| {
        //             if output.remaining_params.contains(&i) {
        //                 Some(p)
        //             } else {
        //                 None
        //             }
        //         }),
        //     ),
        //     shader,
        // }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
)]
pub enum MaterialParameter {
    Uint(u32),
    Int(i32),
    Flt(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Mat4(Mat4),
}
