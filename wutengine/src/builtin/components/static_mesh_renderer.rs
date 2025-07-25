use serde::{Deserialize, Serialize};
use wgpu::VertexAttribute;
use wutengine_asset::AssetHandle;
use wutengine_graphics::material::Material;
use wutengine_graphics::mesh::{Mesh, create_vertex_buffer_layout};
use wutengine_graphics::resource::GpuResource;
use wutengine_graphics::shader::ShaderVertexLayout;

use crate::component::{Component, Renderer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticMeshRenderer {
    mesh: Option<AssetHandle<Mesh>>,
    material: Option<AssetHandle<Material>>,

    #[serde(skip)]
    pipeline: GpuResource<wgpu::RenderPipeline>,
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
            pipeline: GpuResource::new(),
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
    // fn on_render(&mut self, _context: crate::prelude::ComponentContext) {
    //     log::debug!("StaticMeshRenderer on_render");

    //     let (mesh, material) = match (&mut self.mesh, &mut self.material) {
    //         (Some(mesh), Some(mat)) => (mesh, mat),
    //         // Nothing to do
    //         _ => return,
    //     };
    // }
}

impl Renderer for StaticMeshRenderer {
    fn render_color<'a>(
        &mut self,
        pass: &mut wgpu::RenderPass<'a>,
        target_format: wgpu::TextureFormat,
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

        // 16 chosen randomly. Should fit right?
        let shader_layout = if let Some(sh) = material.get_shader() {
            sh.vertex_layout
        } else {
            return;
        };

        let mesh_layout = mesh.get_layout();

        let mut buf = vec![
            VertexAttribute {
                // Fill with random defaults
                format: wgpu::VertexFormat::Uint8,
                offset: 0,
                shader_location: 0
            };
            shader_layout.num_attrs()
        ];

        let vertex_layout = if let Some(layout) =
            create_vertex_buffer_layout(&mut buf, &mesh_layout, &shader_layout)
        {
            layout
        } else {
            log::debug!("Not rendering due to incompatible layout");
            return;
        };

        let pipeline =
            if let Some(pipeline) = material.get_render_pipeline(vertex_layout, target_format) {
                pipeline
            } else {
                return;
            };

        pass.set_pipeline(&pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(index_buffer.slice(..), mesh.get_index_precision().into());
        pass.draw_indexed(0..mesh.num_indices(), 0, 0..1);
    }
}
