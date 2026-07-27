//! Textures that can be used as a render target

/// A set of textures that can be used both as a render target and as a binding in a shader
#[derive(Debug, Clone)]
pub struct RenderTexture {
    /// The color texture
    pub(crate) color: wgpu::Texture,

    /// The depth/stencil texture. Optional
    pub(crate) depthstencil: Option<wgpu::Texture>,
}

impl RenderTexture {
    /// Creates a new [`RenderTexture`] with the given parameters. If `depth_stencil_format` is [`None`],
    /// No depth/stencil texture will be created
    pub fn new(
        size: (u32, u32),
        color_format: wgpu::TextureFormat,
        depth_stentil_format: Option<wgpu::TextureFormat>,
        label: Option<&str>,
    ) -> Self {
        let color_tex = crate::device().create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: color_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let depth_tex = depth_stentil_format.map(|depth_stencil_format| {
            assert!(
                depth_stencil_format.is_depth_stencil_format(),
                "Given depth/stencil format is not actually a depth/stencil format"
            );

            crate::device().create_texture(&wgpu::TextureDescriptor {
                label,
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: depth_stencil_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
        });

        Self {
            color: color_tex,
            depthstencil: depth_tex,
        }
    }

    /// Returns the size (in pixels) of this texture
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.color.size().width, self.color.size().height)
    }

    /// Returns the raw color [`wgpu::Texture`]
    #[inline]
    pub fn color(&self) -> &wgpu::Texture {
        &self.color
    }

    /// Returns the raw depth/stencil [`wgpu::Texture`], if one exists
    #[inline]
    pub fn depth_stencil(&self) -> Option<&wgpu::Texture> {
        self.depthstencil.as_ref()
    }

    /// Destroys the textures used by this [`RenderTexture`]
    pub fn destroy(&self) {
        self.color.destroy();

        if let Some(depthstencil) = self.depthstencil.as_ref() {
            depthstencil.destroy();
        }
    }
}
