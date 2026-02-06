//! GPU Shaders

use std::borrow::Cow;

use wgpu::naga::front::wgsl::Options;

use super::GFX_DEVICE;

/// A generic shader.
pub struct Shader {
    /// The identifier of the shader
    pub id: String,

    /// The raw WGSL source code
    pub source: String,
}

impl Shader {
    pub(crate) fn compile(&self) -> CompiledShader {
        let mut opts = Options::new();
        opts.parse_doc_comments = true;

        let mut frontend = wgpu::naga::front::wgsl::Frontend::new_with_options(opts);

        let compiled = frontend
            .parse(&self.source)
            .expect("Failed to compile shader");

        CompiledShader {
            source: Box::new(wgpu::ShaderSource::Naga(Cow::Owned(compiled))),
            module: None,
        }
    }
}

/// A [Shader] that's been through the WutEngine compilation process, resulting
/// in concrete source code. Can be used in graphics pipelines.
///
/// Note that "compiled" here means that the shader has been compiled by WutEngine only.
/// It still needs to go through compilation in the actual rendering backend.
#[derive(Debug, Clone)]
pub struct CompiledShader {
    pub(super) source: Box<wgpu::ShaderSource<'static>>,

    pub(super) module: Option<wgpu::ShaderModule>,
}

impl CompiledShader {
    pub(crate) fn ensure_compiled(&mut self) {
        if self.module.is_some() {
            return;
        }

        let compiled = GFX_DEVICE.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compiled shader"),
            source: self.source.as_ref().clone(),
        });

        let compinfo = pollster::block_on(compiled.get_compilation_info());

        for message in compinfo.messages {
            dbg!(message);
        }

        self.module = Some(compiled);
    }

    pub(crate) fn get_module(&self) -> &wgpu::ShaderModule {
        self.module
            .as_ref()
            .expect("Compiled shader not yet compiled")
    }
}
