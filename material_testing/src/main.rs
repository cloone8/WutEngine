use core::num::NonZero;
use core::ops::RangeInclusive;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use bind_group::BindGroup;
use glam::{Mat4, Vec2, Vec3, Vec4};
use material_shadercomp::CompInput;
use serde::{Deserialize, Serialize};
use types::{GMat4x4, ShaderBufferParameterType, ShaderOpaqueParameterType};
use wgpu::{BackendOptions, Backends, InstanceFlags, ShaderStages};

mod bind_group;
mod types;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Shader {
    #[serde(skip)]
    id: usize,
    name: String,
    camera_params: bool,
    instance_params: bool,
    keywords: HashMap<String, ShaderKeyword>,
    user_params: Vec<ShaderParameter>,
    source: ShaderSource,
}

impl Shader {
    pub fn load_source(&mut self) {
        if let ShaderSource::File { path } = &self.source {
            let content = std::fs::read_to_string(path).unwrap();
            self.source = ShaderSource::Inline { content };
        }
    }

    pub fn get_source(&self) -> &str {
        if let ShaderSource::Inline { content } = &self.source {
            content.as_str()
        } else {
            panic!("Invalid source");
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ShaderKeyword {
    default: u64,
    allowed: RangeInclusive<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
enum ShaderParameter {
    Buffer {
        #[serde(rename = "type")]
        ty: ShaderBufferParameterType,

        name: String,

        condition: Option<ShaderParameterCondition>,
    },
    Opaque {
        #[serde(rename = "type")]
        ty: ShaderOpaqueParameterType,

        name: String,

        condition: Option<ShaderParameterCondition>,
    },
}

impl ShaderParameter {
    pub fn get_condition(&self) -> Option<&ShaderParameterCondition> {
        match self {
            Self::Buffer { condition, .. } => condition.as_ref(),
            Self::Opaque { condition, .. } => condition.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
enum ShaderSource {
    Inline { content: String },
    File { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
struct ShaderParameterCondition(pub(crate) String);

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
fn main() {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::VULKAN,
        flags: InstanceFlags::advanced_debugging(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: BackendOptions::default(),
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .unwrap();

    let adapter_info = adapter.get_info();

    println!(
        "Using graphics device '{}' with backend '{}' and driver '{} {}'",
        adapter_info.name, adapter_info.backend, adapter_info.driver, adapter_info.driver_info
    );

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("WutEngine Main GPU"),
        ..Default::default()
    }))
    .unwrap();

    let camera_params = [
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
    ];

    let camera_layout: wgpu::BindGroupLayout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZero::new(BindGroup::total_buffer_size(params_to_buf_iter(
                            &camera_params,
                        )) as u64)
                        .unwrap(),
                    ),
                },
                count: None,
            }],
        });

    let mut camera_bind_group = BindGroup::new("Camera".to_string(), camera_layout, &camera_params);

    let instance_params = [
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
    ];

    let instance_layout: wgpu::BindGroupLayout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Instance layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(
                        NonZero::new(BindGroup::total_buffer_size(params_to_buf_iter(
                            &instance_params,
                        )) as u64)
                        .unwrap(),
                    ),
                },
                count: None,
            }],
        });

    let mut instance_bind_group =
        BindGroup::new("Instance".to_string(), instance_layout, &instance_params);

    let mut desc: Shader = serde_json::from_str(include_str!("unlit.json")).unwrap();

    println!("{:#?}", desc);

    desc.load_source();

    let mut keywords = HashMap::default();

    keywords.insert("HAS_COLOR_MAP".to_owned(), 0);

    let material = Material::new(desc, keywords, &device);

    dbg!(material);
}

#[derive(Debug)]
struct CompiledShader {
    pub module: Box<naga::Module>,
    pub source_id_hash: u64,
    pub keyword_hash: u64,
    pub user_bind_group_layout: wgpu::BindGroupLayout,
}

#[derive(Debug)]
struct Material {
    shader: Shader,
    keywords: HashMap<String, u64>,
    compiled: CompiledShader,
    user_bind_group: BindGroup,
}

impl Material {
    pub fn new(shader: Shader, keywords: HashMap<String, u64>, device: &wgpu::Device) -> Self {
        //TODO: Check cache

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

        let compiled = CompiledShader {
            module: output.module,
            source_id_hash: output.source_id_hash,
            keyword_hash: output.keyword_hash,
            user_bind_group_layout: user_bind_group_layout.clone(),
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
        visibility: ShaderStages::VERTEX_FRAGMENT,
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
            visibility: ShaderStages::VERTEX_FRAGMENT,
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

struct FakeHasher;

impl material_shadercomp::ShaderHasher<u64> for FakeHasher {
    fn hash_source_id(id: u64) -> u64 {
        id
    }

    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64 {
        let mut kw_vec: Vec<(&str, u64)> = keywords.iter().map(|(k, v)| (k.as_ref(), *v)).collect();
        kw_vec.sort();

        let mut hash: u64 = 0;

        for (k, v) in kw_vec {
            for x in k.as_bytes().iter().map(|b| *b as u64) {
                hash = hash.wrapping_add(x);
            }

            hash = hash.wrapping_add(v)
        }

        hash
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
