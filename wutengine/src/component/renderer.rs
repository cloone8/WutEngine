use super::Component;

pub trait Renderer: Component {
    fn render_color<'a>(
        &mut self,
        encoder: &mut crate::graphics::wgpu::RenderPass<'a>,
        target_format: crate::graphics::wgpu::TextureFormat,
    );
}
