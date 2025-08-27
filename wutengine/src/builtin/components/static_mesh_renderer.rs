use serde::{Deserialize, Serialize};
use wutengine_asset::AssetHandle;
use wutengine_graphics::material::Material;
use wutengine_graphics::mesh::{Mesh, create_vertex_buffer_layout};
use wutengine_graphics::pipeline::cache::PipelineCacheKey;
use wutengine_graphics::wgpu::VertexAttribute;

use crate::component::{Component, ComponentCallbacks, renderer::Renderer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticMeshRenderer {
    mesh: Option<AssetHandle<Mesh>>,
    material: Option<AssetHandle<Material>>,
}

impl Default for StaticMeshRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl StaticMeshRenderer {
    pub fn new() -> Self {
        Self {
            mesh: None,
            material: None,
        }
    }

    pub fn set_mesh(&mut self, mesh: Option<impl Into<AssetHandle<Mesh>>>) {
        self.mesh = mesh.map(|m| m.into());
    }

    pub fn set_material(&mut self, material: Option<impl Into<AssetHandle<Material>>>) {
        self.material = material.map(|m| m.into());
    }
}

impl Component for StaticMeshRenderer {
    fn as_renderer(&mut self) -> Option<&mut dyn Renderer> {
        Some(self)
    }

    fn wanted_callbacks() -> crate::component::ComponentCallbacks
    where
        Self: Sized,
    {
        ComponentCallbacks::empty()
    }
}

impl Renderer for StaticMeshRenderer {
    fn render_color<'a>(
        &mut self,
        pass: &mut crate::graphics::wgpu::RenderPass<'a>,
        target_format: crate::graphics::wgpu::TextureFormat,
    ) {
        let (mesh, material) = if let (Some(mesh), Some(material)) = (&self.mesh, &self.material) {
            (mesh, material)
        } else {
            return;
        };

        let vertex_buffer = if let Some(buf) = mesh.get_vertex_buffer() {
            buf
        } else {
            return;
        };

        let index_buffer = if let Some(buf) = mesh.get_index_buffer() {
            buf
        } else {
            return;
        };

        let Some(shader) = material.get_shader() else {
            return;
        };

        let pipeline_cache_key = PipelineCacheKey {
            shader: shader.name.clone(),
            shader_keyword_hash: material.get_keyword_hash(),
            mesh_layout: mesh.get_layout(),
        };

        let pipeline =
            match wutengine_graphics::pipeline::cache::get_cached_pipeline(&pipeline_cache_key) {
                Some(cached) => cached,
                None => {
                    let mut buf = vec![
                        VertexAttribute {
                            // Fill with random defaults
                            format: crate::graphics::wgpu::VertexFormat::Uint8,
                            offset: 0,
                            shader_location: 0
                        };
                        shader.vertex_layout.num_attrs()
                    ];
                    let vertex_layout = if let Some(layout) = create_vertex_buffer_layout(
                        &mut buf,
                        &pipeline_cache_key.mesh_layout,
                        &shader.vertex_layout,
                    ) {
                        layout
                    } else {
                        log::debug!("Not rendering due to incompatible layout");
                        return;
                    };

                    let Some(pipeline) = material.get_render_pipeline(vertex_layout, target_format)
                    else {
                        log::warn!("Could not create material render pipeline");
                        return;
                    };

                    wutengine_graphics::pipeline::cache::cache_pipeline(
                        pipeline_cache_key,
                        pipeline,
                    )
                }
            };

        pass.set_pipeline(&pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(index_buffer.slice(..), mesh.get_index_precision().into());
        pass.draw_indexed(0..mesh.num_indices(), 0, 0..1);
    }
}
