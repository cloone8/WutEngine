//! Shader resource binding discovery

use naga::GlobalVariable;
use naga::TypeInner;
use wutengine_assets::assets::shader::Binding;
use wutengine_assets::assets::shader::BindingKind;
use wutengine_assets::assets::shader::BufferBaseType;
use wutengine_assets::assets::shader::BufferMember;
use wutengine_assets::assets::shader::BufferSpace;
use wutengine_assets::assets::shader::TextureDimension;
use wutengine_assets::assets::shader::TextureSampleType;

/// An error while trying to find shader bindings with [`find_bindings`]
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum FindBindingsErr {
    /// Nameless global
    #[display("Found a resource global without a name: {_0:#?}")]
    Nameless(#[error(not(source))] Box<GlobalVariable>),

    /// Global without binding
    #[display("Resource global `{_0}` has no @group/@binding")]
    MissingBinding(#[error(not(source))] String),

    /// Unsupported type
    #[display("Type of resource global `{name}` is not yet supported: {ty:#?}")]
    UnsupportedType {
        /// The name of the global or buffer member
        name: String,

        /// The type
        #[error(not(source))]
        ty: Box<TypeInner>,
    },

    /// Non-concrete array size
    #[display("Array has an unknown/non-concrete size: {_0}")]
    UnknownArraySize(#[error(not(source))] String),
}

/// Finds the resource bindings (uniform and storage buffers, textures and samplers) in a [`naga::Module`], skipping
/// globals for which `is_used` returns `false`
pub fn find_bindings(
    module: &naga::Module,
    is_used: impl Fn(naga::Handle<GlobalVariable>) -> bool,
) -> Result<Vec<Binding>, FindBindingsErr> {
    profiling::function_scope!();
    log::debug!("Finding bindings for module");

    let mut bindings = Vec::new();

    for (handle, global) in module.global_variables.iter() {
        if !is_used(handle) {
            continue;
        }

        let space = match global.space {
            naga::AddressSpace::Uniform => Some(BufferSpace::Uniform),
            naga::AddressSpace::Storage { access } => Some(BufferSpace::Storage {
                writable: access.contains(naga::StorageAccess::STORE),
            }),
            naga::AddressSpace::Handle => None,
            // Not a resource the engine binds
            _ => continue,
        };

        let name = global
            .name
            .clone()
            .ok_or_else(|| FindBindingsErr::Nameless(Box::new(global.clone())))?;

        let res_binding = global
            .binding
            .ok_or_else(|| FindBindingsErr::MissingBinding(name.clone()))?;

        let ty = &module.types[global.ty].inner;

        let kind = match (space, ty) {
            (Some(space), TypeInner::Struct { members, span }) => BindingKind::Buffer {
                space,
                size: *span,
                members: members
                    .iter()
                    .map(|member| {
                        map_buffer_member(
                            module,
                            member.name.clone().unwrap_or_default(),
                            member.ty,
                            member.offset,
                        )
                    })
                    .collect::<Result<_, _>>()?,
            },
            (Some(space), _) => BindingKind::Buffer {
                space,
                size: ty.size(module.to_ctx()),
                members: vec![map_buffer_member(module, name.clone(), global.ty, 0)?],
            },
            (None, TypeInner::Sampler { comparison }) => BindingKind::Sampler {
                comparison: *comparison,
            },
            (
                None,
                TypeInner::Image {
                    dim,
                    arrayed,
                    class,
                },
            ) => {
                let (sample_type, multisampled) = match class {
                    naga::ImageClass::Sampled { kind, multi } => (
                        match kind {
                            naga::ScalarKind::Sint => TextureSampleType::Sint,
                            naga::ScalarKind::Uint => TextureSampleType::Uint,
                            _ => TextureSampleType::Float,
                        },
                        *multi,
                    ),
                    naga::ImageClass::Depth { multi } => (TextureSampleType::Depth, *multi),
                    naga::ImageClass::External | naga::ImageClass::Storage { .. } => {
                        return Err(unsupported(name, ty));
                    }
                };

                BindingKind::Texture {
                    dimension: match dim {
                        naga::ImageDimension::D1 => TextureDimension::D1,
                        naga::ImageDimension::D2 => TextureDimension::D2,
                        naga::ImageDimension::D3 => TextureDimension::D3,
                        naga::ImageDimension::Cube => TextureDimension::Cube,
                    },
                    arrayed: *arrayed,
                    sample_type,
                    multisampled,
                }
            }
            (None, other) => return Err(unsupported(name, other)),
        };

        bindings.push(Binding {
            name,
            group: res_binding.group,
            binding: res_binding.binding,
            kind,
        });
    }

    Ok(bindings)
}

/// Builds an [`FindBindingsErr::UnsupportedType`]
fn unsupported(name: String, ty: &TypeInner) -> FindBindingsErr {
    FindBindingsErr::UnsupportedType {
        name,
        ty: Box::new(ty.clone()),
    }
}

/// Maps a value in a buffer to a [`BufferMember`]
fn map_buffer_member(
    module: &naga::Module,
    name: String,
    ty: naga::Handle<naga::Type>,
    offset: u32,
) -> Result<BufferMember, FindBindingsErr> {
    let mut member_type = &module.types[ty].inner;
    let size = member_type.size(module.to_ctx());

    log::trace!("Base type of `{name}`:\n{member_type:#?}");

    let mut array_length = 1;

    if let TypeInner::Array { base, size, .. } = member_type {
        member_type = &module.types[*base].inner;

        array_length = get_concrete_size(*size)
            .ok_or_else(|| FindBindingsErr::UnknownArraySize(name.clone()))?;
    }

    let base_type = match member_type {
        TypeInner::Matrix { columns, rows, .. } => base_type_from_matrix(*columns, *rows),
        TypeInner::Array { size, .. } => {
            // As we've already removed one layer of array indirection, we now see this inner type as "complex"
            get_concrete_size(*size)
                .ok_or_else(|| FindBindingsErr::UnknownArraySize(name.clone()))?;

            BufferBaseType::Complex
        }
        TypeInner::Struct { .. } => BufferBaseType::Complex,
        TypeInner::Scalar(scalar) => base_type_from_scalar(*scalar),
        TypeInner::Vector { size, scalar } => base_type_from_vector(*scalar, *size),
        other => return Err(unsupported(name, other)),
    };

    Ok(BufferMember {
        name,
        base_type,
        array_length,
        offset,
        size,
    })
}

const fn get_concrete_size(size: naga::ArraySize) -> Option<u32> {
    match size {
        naga::ArraySize::Constant(non_zero) => Some(non_zero.get()),
        naga::ArraySize::Pending(_) => None,
        naga::ArraySize::Dynamic => None,
    }
}

const fn base_type_from_scalar(scalar: naga::Scalar) -> BufferBaseType {
    match scalar.kind {
        naga::ScalarKind::Sint => BufferBaseType::Sint,
        naga::ScalarKind::Uint => BufferBaseType::Uint,
        naga::ScalarKind::Float => BufferBaseType::Float,
        naga::ScalarKind::Bool => BufferBaseType::Bool,
        naga::ScalarKind::AbstractInt => unreachable!(),
        naga::ScalarKind::AbstractFloat => unreachable!(),
    }
}

const fn base_type_from_vector(scalar: naga::Scalar, vecsize: naga::VectorSize) -> BufferBaseType {
    match (scalar.kind, vecsize) {
        (naga::ScalarKind::Sint, naga::VectorSize::Bi) => BufferBaseType::Vec2i,
        (naga::ScalarKind::Sint, naga::VectorSize::Tri) => BufferBaseType::Vec3i,
        (naga::ScalarKind::Sint, naga::VectorSize::Quad) => BufferBaseType::Vec4i,
        (naga::ScalarKind::Uint, naga::VectorSize::Bi) => BufferBaseType::Vec2u,
        (naga::ScalarKind::Uint, naga::VectorSize::Tri) => BufferBaseType::Vec3u,
        (naga::ScalarKind::Uint, naga::VectorSize::Quad) => BufferBaseType::Vec4u,
        (naga::ScalarKind::Float, naga::VectorSize::Bi) => BufferBaseType::Vec2f,
        (naga::ScalarKind::Float, naga::VectorSize::Tri) => BufferBaseType::Vec3f,
        (naga::ScalarKind::Float, naga::VectorSize::Quad) => BufferBaseType::Vec4f,
        (naga::ScalarKind::Bool, naga::VectorSize::Bi) => BufferBaseType::Vec2b,
        (naga::ScalarKind::Bool, naga::VectorSize::Tri) => BufferBaseType::Vec3b,
        (naga::ScalarKind::Bool, naga::VectorSize::Quad) => BufferBaseType::Vec4b,
        (naga::ScalarKind::AbstractInt, _) => unreachable!(),
        (naga::ScalarKind::AbstractFloat, _) => unreachable!(),
    }
}

const fn base_type_from_matrix(cols: naga::VectorSize, rows: naga::VectorSize) -> BufferBaseType {
    match (cols, rows) {
        (naga::VectorSize::Bi, naga::VectorSize::Bi) => BufferBaseType::Mat2x2,
        (naga::VectorSize::Bi, naga::VectorSize::Tri) => BufferBaseType::Mat2x3,
        (naga::VectorSize::Bi, naga::VectorSize::Quad) => BufferBaseType::Mat2x4,
        (naga::VectorSize::Tri, naga::VectorSize::Bi) => BufferBaseType::Mat3x2,
        (naga::VectorSize::Tri, naga::VectorSize::Tri) => BufferBaseType::Mat3x3,
        (naga::VectorSize::Tri, naga::VectorSize::Quad) => BufferBaseType::Mat3x4,
        (naga::VectorSize::Quad, naga::VectorSize::Bi) => BufferBaseType::Mat4x2,
        (naga::VectorSize::Quad, naga::VectorSize::Tri) => BufferBaseType::Mat4x3,
        (naga::VectorSize::Quad, naga::VectorSize::Quad) => BufferBaseType::Mat4x4,
    }
}
