use core::fmt::Display;
use std::collections::HashMap;

/// The descriptor for a single generic [Shader] uniform, used by WutEngine
/// graphics backends to properly map data to their shaders
#[derive(Debug, Clone)]
pub struct Uniform {
    /// The uniform type
    pub ty: UniformType,

    /// The uniform "binding". This refers to the actual binding in the shader
    pub binding: UniformBinding,
}

/// The type of a [Uniform]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UniformType {
    /// An unsigned integer value
    U32,

    /// A three-f32 vector
    Vec3,

    /// A four-f32 vector
    Vec4,

    /// A 4x4 f32 matrix
    Mat4,

    /// A 2D texture
    Tex2D,

    /// A struct
    Struct(HashMap<String, UniformType>),

    /// An array
    Array(Box<UniformType>, usize),
}

impl Display for UniformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniformType::U32 => write!(f, "u32"),
            UniformType::Vec3 => write!(f, "vec3<f32>"),
            UniformType::Vec4 => write!(f, "vec4<f32>"),
            UniformType::Mat4 => write!(f, "mat4x4<f32>"),
            UniformType::Tex2D => write!(f, "texture_2d<f32>"),
            UniformType::Struct(hash_map) => {
                for (k, v) in hash_map {
                    writeln!(f, "{}: {}", k, v)?;
                }

                Ok(())
            }
            UniformType::Array(uniform_type, len) => {
                write!(f, "array<{}, {}>", uniform_type, len)
            }
        }
    }
}

impl UniformType {
    /// Returns whether this [UniformType] is any type of texture
    pub const fn is_texture_type(&self) -> bool {
        matches!(self, Self::Tex2D)
    }
}

/// A binding description for a [Uniform]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UniformBinding {
    /// A single binding
    Standard(SingleUniformBinding),

    /// A texture binding. Some backends have seperate bindings for the
    /// texture and the sampler, some do not.
    Texture {
        /// The sampler part of the texture
        sampler: Option<SingleUniformBinding>,

        /// The texture part of the texture
        texture: Option<SingleUniformBinding>,
    },
}

impl Display for UniformBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniformBinding::Standard(s) => write!(f, "uniform({})", s),
            UniformBinding::Texture { sampler, texture } => match (sampler, texture) {
                (None, None) => write!(f, "texture_uniform(<none>)"),
                (None, Some(t)) => {
                    write!(f, "texture_uniform(sampler: <none>, texture: {})", t)
                }
                (Some(s), None) => {
                    write!(f, "texture_uniform(sampler: {}, texture: <none>)", s)
                }
                (Some(s), Some(t)) => {
                    write!(f, "texture_uniform(sampler: {}, texture: {})", s, t)
                }
            },
        }
    }
}

impl UniformBinding {
    pub fn try_as_standard(&self) -> Option<&SingleUniformBinding> {
        if let Self::Standard(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_standard(&self) -> &SingleUniformBinding {
        self.try_as_standard()
            .expect("Uniform was not a standard uniform binding")
    }

    pub fn try_as_texture(
        &self,
    ) -> Option<(Option<&SingleUniformBinding>, Option<&SingleUniformBinding>)> {
        if let Self::Texture { sampler, texture } = self {
            Some((sampler.as_ref(), texture.as_ref()))
        } else {
            None
        }
    }

    pub fn as_texture(&self) -> (Option<&SingleUniformBinding>, Option<&SingleUniformBinding>) {
        self.try_as_texture()
            .expect("Uniform was not a texture uniform binding")
    }
}

/// The shader source binding for a [Uniform]. A combination of any/all of these
/// values is used by the graphics backend
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SingleUniformBinding {
    /// The name of the uniform in the shader
    pub name: String,

    /// The uniform group
    pub group: usize,

    /// The uniform binding
    pub binding: usize,
}

impl Display for SingleUniformBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{{} @ group({}) binding({})}}",
            self.name, self.group, self.binding
        )
    }
}
