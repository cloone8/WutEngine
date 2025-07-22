use std::collections::HashMap;

use naga::keywords;
use serde::{Deserialize, Serialize};
use wgpu::{
    PipelineCompilationOptions, PipelineLayoutDescriptor, RenderPipelineDescriptor,
    ShaderModuleDescriptor,
};
use wutengine_asset::{Asset, AssetHandle};
use wutengine_shadercompiler::{CompileStage, ShaderOutput};

use crate::GRAPHICS_MANAGER;
use crate::resource::GpuResource;
use crate::shader::{CompiledShader, ShaderSource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    shader: Option<ShaderVariant>,
    keywords: HashMap<String, i64>,
}

impl Asset for Material {}

impl Material {
    pub(crate) fn get_pipeline_layout(&self) -> wgpu::PipelineLayout {
        let layout = GRAPHICS_MANAGER
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Material pipeline"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        layout
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderVariant {
    pub source: AssetHandle<ShaderSource>,
    pub keywords: HashMap<String, i64>,
    pub compiled: Option<AssetHandle<CompiledShader>>,
}

impl ShaderVariant {
    #[profiling::function]
    pub fn ensure_compiled(&mut self) -> Result<(), wutengine_shadercompiler::Error> {
        if self.compiled.is_none() {
            log::debug!("Compiling shader variant");

            let keywords = HashMap::from_iter(self.keywords.iter().map(|(k, v)| (k.as_ref(), *v)));

            let compile_result = wutengine_shadercompiler::compile_single_shader(
                &self.source.code,
                keywords,
                CompileStage::Full,
            )?;

            if let ShaderOutput::Compiled {
                source,
                keyword_hash,
                keywords: _,
            } = compile_result
            {
                self.compiled = Some(AssetHandle::from(CompiledShader {
                    name: self.source.name.clone(),
                    keyword_hash,
                    renderer_data: GpuResource::default(),
                    source: *source,
                }));
            } else {
                unreachable!("Shader is fully compiled after compilation");
            }
        }

        let compiled = self
            .compiled
            .as_mut()
            .expect("Shader should have been compiled now");

        if compiled.renderer_data.is_loaded() {
            return Ok(());
        }

        {
            profiling::scope!("create_native_shader_module");

            let keyword_hash = compiled.keyword_hash;
            let source = compiled.source.clone();

            compiled
                .renderer_data
                .set(
                    GRAPHICS_MANAGER
                        .device
                        .create_shader_module(ShaderModuleDescriptor {
                            label: Some(
                                format!("{}::{:032x}", &self.source.name, keyword_hash).as_str(),
                            ),
                            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(source)),
                        }),
                );
        }

        Ok(())
    }

    pub(crate) fn get_shader_module(&self) -> Option<&wgpu::ShaderModule> {
        self.compiled
            .as_ref()
            .and_then(|compiled| compiled.renderer_data.get())
    }
}
