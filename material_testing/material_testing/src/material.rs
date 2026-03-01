use core::num::NonZero;
use std::collections::{HashMap, HashSet};

use glam::{Mat4, Vec2, Vec3, Vec4};
use material_shadercomp::CompInput;

use crate::bind_group::BindGroup;
use crate::{
    CompiledShader, FakeHasher, Shader, ShaderParameter, ShaderVertexAttributeType,
    params_to_buf_iter,
};

#[derive(Debug)]
pub(crate) struct Material {
    pub(crate) shader: Shader,
    pub(crate) keywords: HashMap<String, u64>,
    pub(crate) compiled: CompiledShader,
    pub(crate) user_bind_group: BindGroup,
}

impl Material {
    pub(crate) fn new(
        shader: Shader,
        keywords: HashMap<String, u64>,
        device: &wgpu::Device,
    ) -> Self {
        //TODO: Check cache

        let vertex_attr_conditions: Vec<Option<&str>> = Vec::from_iter(
            shader
                .vertex_attributes
                .iter()
                .map(|p| p.condition.as_ref().map(|c| c.0.as_str())),
        );

        let user_param_conditions: Vec<Option<&str>> = Vec::from_iter(
            shader
                .user_params
                .iter()
                .map(|p| p.get_condition().map(|c| c.0.as_str())),
        );

        let output = material_shadercomp::compile::<_, FakeHasher>(CompInput {
            id: shader.id as u64,
            source: shader.get_source(),
            keywords: &keywords,
            vertex_attributes: &vertex_attr_conditions,
            user_params: &user_param_conditions,
            per_camera_block: include_str!("camera.wgsl"),
            per_instance_block: include_str!("instance.wgsl"),
        })
        .unwrap();

        let user_bind_group_layout: wgpu::BindGroupLayout = create_user_params_bind_group_layout(
            "Material BGL",
            &shader.user_params,
            &output.remaining_params,
            device,
        );

        let vertex_attributes = output
            .remaining_vertex_attributes
            .into_iter()
            .map(|idx| {
                let attr = &shader.vertex_attributes[idx];
                (attr.ty, attr.location)
            })
            .collect();

        let compiled = CompiledShader {
            module: output.module,
            source_id_hash: output.source_id_hash,
            keyword_hash: output.keyword_hash,
            user_bind_group_layout: user_bind_group_layout.clone(),
            vertex_attributes,
        };

        Self {
            keywords,
            compiled,
            user_bind_group: BindGroup::new(
                "Material User Bind Group".to_string(),
                user_bind_group_layout,
                shader.user_params.iter().enumerate().filter_map(|(i, p)| {
                    if output.remaining_params.contains(&i) {
                        Some(p)
                    } else {
                        None
                    }
                }),
            ),
            shader,
        }
    }
}

fn create_user_params_bind_group_layout(
    name: &str,
    params: &[ShaderParameter],
    after_compile_filter: &HashSet<usize>,
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    let params_with_filter = params
        .iter()
        .enumerate()
        .filter(|(index, _)| after_compile_filter.contains(index))
        .map(|(_, p)| p);

    let buffer_entry = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(
                NonZero::new(BindGroup::total_buffer_size(params_to_buf_iter(
                    params_with_filter.clone(),
                )) as u64)
                .unwrap(),
            ),
        },
        count: None,
    };

    let mut all_entries = vec![buffer_entry];

    for param in params_with_filter {
        let ShaderParameter::Opaque { ty, .. } = param else {
            continue;
        };

        let binding = all_entries.len();

        let opaque_entry = wgpu::BindGroupLayoutEntry {
            binding: binding as u32,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: ty.to_wgpu_binding_type(),
            count: None,
        };

        all_entries.push(opaque_entry);
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(name),
        entries: &all_entries,
    })
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
