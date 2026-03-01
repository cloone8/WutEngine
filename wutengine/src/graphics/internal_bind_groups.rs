use core::num::NonZero;
use std::sync::LazyLock;

use super::shader::{ShaderBufferParameterType, ShaderParameter};
use super::{BindGroup, GFX_DEVICE};

fn get_camera_params() -> &'static [ShaderParameter] {
    static CAMERA_PARAMS: LazyLock<[ShaderParameter; 3]> = LazyLock::new(|| {
        [
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "view".to_string(),
                condition: None,
            },
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "projection".to_string(),
                condition: None,
            },
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "vp".to_string(),
                condition: None,
            },
        ]
    });

    &*CAMERA_PARAMS
}

pub(crate) fn get_camera_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    static CAMERA_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
        let params = get_camera_params();

        GFX_DEVICE.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZero::new(
                            BindGroup::total_buffer_size(params_to_buf_iter(params)) as u64
                        )
                        .unwrap(),
                    ),
                },
                count: None,
            }],
        })
    });

    &*CAMERA_LAYOUT
}

pub(crate) fn create_camera_bind_group(name: String) -> BindGroup {
    BindGroup::new(
        name,
        get_camera_bind_group_layout().clone(),
        get_camera_params(),
    )
}

fn get_instance_params() -> &'static [ShaderParameter] {
    static INSTANCE_PARAMS: LazyLock<[ShaderParameter; 2]> = LazyLock::new(|| {
        [
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "model".to_string(),
                condition: None,
            },
            ShaderParameter::Buffer {
                ty: ShaderBufferParameterType::Mat4x4,
                name: "mvp".to_string(),
                condition: None,
            },
        ]
    });

    &*INSTANCE_PARAMS
}

pub(crate) fn get_instance_bind_group_layout() -> &'static wgpu::BindGroupLayout {
    static INSTANCE_LAYOUT: LazyLock<wgpu::BindGroupLayout> = LazyLock::new(|| {
        let params = get_instance_params();

        GFX_DEVICE.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Instance layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZero::new(
                            BindGroup::total_buffer_size(params_to_buf_iter(params)) as u64
                        )
                        .unwrap(),
                    ),
                },
                count: None,
            }],
        })
    });

    &*INSTANCE_LAYOUT
}

pub(crate) fn create_instance_bind_group(name: String) -> BindGroup {
    BindGroup::new(
        name,
        get_instance_bind_group_layout().clone(),
        get_instance_params(),
    )
}

fn params_to_buf_iter<'a>(
    params: impl IntoIterator<Item = &'a ShaderParameter>,
) -> impl IntoIterator<Item = ShaderBufferParameterType> {
    params.into_iter().filter_map(|p| {
        if let ShaderParameter::Buffer { ty, .. } = p {
            Some(*ty)
        } else {
            None
        }
    })
}
