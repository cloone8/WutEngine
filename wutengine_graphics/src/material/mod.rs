use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use naga::keywords;
use serde::{Deserialize, Serialize};
use wgpu::{
    BindGroupLayoutDescriptor, BlendState, ColorWrites, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, RenderPipelineDescriptor,
    ShaderModuleDescriptor, TextureFormat, VertexBufferLayout,
};
use wutengine_asset::{Asset, AssetHandle};
use wutengine_shadercompiler::{CompileStage, ShaderOutput};

use crate::GRAPHICS_MANAGER;
use crate::resource::GpuResource;
use crate::shader::{CompiledShader, ShaderConstants, ShaderSource};

fn empty_layout() -> Arc<wgpu::BindGroupLayout> {
    static EMPTY_LAYOUT: RwLock<Option<Arc<wgpu::BindGroupLayout>>> = RwLock::new(None);

    let layout = EMPTY_LAYOUT.read().unwrap();

    match &*layout {
        Some(layout) => layout.clone(),
        None => {
            drop(layout);

            let mut layout_write = EMPTY_LAYOUT.write().unwrap();

            let new_empty = Arc::new(GRAPHICS_MANAGER.device.create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    label: Some("Empty group layout"),
                    entries: &[],
                },
            ));

            *layout_write = Some(new_empty.clone());

            new_empty
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    shader: Option<AssetHandle<ShaderSource>>,
    shader_keywords: HashMap<String, i64>,
    shader_keyword_hash: u64,
}

impl Asset for Material {}

impl Material {
    pub fn get_pipeline_layout(&self) -> Option<wgpu::PipelineLayout> {
        let shader = self.shader.as_ref()?;
        let empty_layout = empty_layout();

        let bind_group_layouts: [Arc<wgpu::BindGroupLayout>; 3] =
            core::array::from_fn(|i| match i {
                0 => {
                    if shader.constants.viewport {
                        ShaderConstants::viewport_bind_group_layout().clone()
                    } else {
                        empty_layout.clone()
                    }
                }
                1 => {
                    //TODO: Actual material parameters here
                    empty_layout.clone()
                }
                2 => {
                    if shader.constants.instance {
                        ShaderConstants::instance_bind_group_layout().clone()
                    } else {
                        empty_layout.clone()
                    }
                }
                _ => unreachable!(),
            });

        let layout = GRAPHICS_MANAGER
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Material pipeline layout"),
                bind_group_layouts: &Vec::from_iter(bind_group_layouts.iter().map(|x| x.as_ref())),
                push_constant_ranges: &[],
            });

        Some(layout)
    }

    pub fn get_render_pipeline(
        &self,
        vertex_layout: VertexBufferLayout,
        target_format: TextureFormat,
    ) -> Option<wgpu::RenderPipeline> {
        let shader = self.shader.as_ref()?;

        let module = crate::shader::get(shader, &self.shader_keywords, self.shader_keyword_hash)?;

        let pipeline = GRAPHICS_MANAGER
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Material renderpipeline"),
                layout: Some(
                    &self
                        .get_pipeline_layout()
                        .expect("Could not create pipeline layout"),
                ),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: None,
                    compilation_options: Default::default(),
                    buffers: &[vertex_layout],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: None,
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::all(),
                    })],
                }),
                primitive: PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        Some(pipeline)
    }

    // pub(crate) fn get_render_pipeline(&self) -> wgpu::RenderPipeline {
    //     let module = self.shader.as_ref().unwrap().get_shader_module().unwrap();

    //     let pipeline = GRAPHICS_MANAGER
    //         .device
    //         .create_render_pipeline(&RenderPipelineDescriptor {
    //             label: Some("Material render pipeline"),
    //             layout: Some(&self.get_pipeline_layout()),
    //             vertex: wgpu::VertexState {
    //                 module: &module,
    //                 entry_point: Some("vertex_main"),
    //                 compilation_options: PipelineCompilationOptions::default(),
    //                 buffers: ,
    //             },
    //             primitive: todo!(),
    //             depth_stencil: todo!(),
    //             multisample: todo!(),
    //             fragment: todo!(),
    //             multiview: todo!(),
    //             cache: todo!(),
    //         });
    // }

    fn rehash_keywords(&mut self) {
        self.shader_keyword_hash = wutengine_util::hash::keyword_hash(&self.shader_keywords);
    }

    fn discard_invalid_keywords(&mut self) {
        let shader = if let Some(shader) = &self.shader {
            shader
        } else {
            return;
        };

        //TODO: Discard invalid

        self.rehash_keywords();
    }
}

/// Public API for a [Material]
impl Material {
    pub fn new() -> Self {
        Material {
            shader: None,
            shader_keywords: HashMap::default(),
            shader_keyword_hash: 0,
        }
    }

    pub fn get_shader(&self) -> Option<&AssetHandle<ShaderSource>> {
        self.shader.as_ref()
    }

    pub fn get_keywords(&self) -> &HashMap<String, i64> {
        &self.shader_keywords
    }

    pub fn get_keyword_hash(&self) -> u64 {
        self.shader_keyword_hash
    }

    pub fn set_shader(&mut self, shader: Option<impl Into<AssetHandle<ShaderSource>>>) {
        self.shader = shader.map(Into::into);

        self.discard_invalid_keywords();
    }

    pub fn set_keyword(&mut self, keyword: &str, value: i64) {
        if let Some(shader) = &self.shader {
            // If we have a shader set, we can check for keyword values
            match shader.available_keywords.get(keyword) {
                Some(possible_values) => {
                    if !possible_values.contains(&value) {
                        log::error!(
                            "Value {} is out of range for keyword {keyword} with possible values {}..={}",
                            value,
                            possible_values.start(),
                            possible_values.end()
                        );
                        return;
                    }
                }
                None => {
                    log::error!("Keyword {keyword} does not exist on shader {}", shader.name);
                    return;
                }
            }
        }

        let cur_val = self.shader_keywords.get_mut(keyword);

        match cur_val {
            Some(cur) => {
                *cur = value;
            }
            None => {
                self.shader_keywords.insert(keyword.to_string(), value);
            }
        };

        self.rehash_keywords();
    }

    pub fn unset_keyword(&mut self, keyword: &str) {
        self.shader_keywords.remove(keyword);
        self.rehash_keywords();
    }
}
