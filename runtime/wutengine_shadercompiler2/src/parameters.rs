//! Shader parameter mapping

/// Information on an exposed parameter in a shader
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Parameter {
    /// An opaque parameter, requiring a resource binding
    Opaque {
        /// The name of the parameter
        name: String,

        /// The parameter group
        group: u32,
        /// The parameter binding
        binding: u32,

        /// The base type
        base_type: OpaqueBaseType,
    },

    /// An in-buffer parameter, residing within a location in a buffer
    BufferMember {
        /// The name of the parameter
        name: String,

        /// The parameter group
        group: u32,
        /// The parameter binding
        binding: u32,

        /// The base type
        base_type: BufferBaseType,

        /// The size in bytes of the base type
        base_size: u32,

        /// If an array, the array length
        array_length: u32,

        /// The offset within the buffer (in bytes)
        offset: u32,

        /// The size of the member in the buffer (in bytes)
        size: u32,
    },
}

/// Base types for opaque parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpaqueBaseType {
    /// A sampler
    Sampler,

    /// An image
    Image,
}

/// Base types for buffer parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BufferBaseType {
    /// A complex, unsupported type
    Complex = 0,

    /// Float
    Float,

    /// Signed integer
    Sint,

    /// Unsigned integer
    Uint,

    /// Boolean
    Bool,

    /// 2-component float vector
    Vec2f,
    /// 2-component float vector
    Vec3f,
    /// 4-component float vector
    Vec4f,

    /// 2-component signed int vector
    Vec2i,
    /// 3-component signed int vector
    Vec3i,
    /// 4-component signed int vector
    Vec4i,

    /// 2-component unsigned int vector
    Vec2u,
    /// 3-component unsigned int vector
    Vec3u,
    /// 4-component unsigned int vector
    Vec4u,

    /// 2-component bool vector
    Vec2b,
    /// 3-component bool vector
    Vec3b,
    /// 4-component bool vector
    Vec4b,

    /// 2x2 float vector
    Mat2x2,
    /// 2x3 float vector
    Mat2x3,
    /// 2x4 float vector
    Mat2x4,
    /// 3x2 float vector
    Mat3x2,
    /// 3x3 float vector
    Mat3x3,
    /// 3x4 float vector
    Mat3x4,
    /// 4x2 float vector
    Mat4x2,
    /// 4x3 float vector
    Mat4x3,
    /// 4x4 float vector
    Mat4x4,
}

/// Find the exposed parameters in a [`naga::Module`]
pub fn find_parameters(module: &naga::Module) -> Vec<Parameter> {
    log::debug!("Finding parameters for module");

    let mut params = Vec::new();

    for (_, global) in module.global_variables.iter() {
        let name = global.name.clone().expect("No name found");
        let res_binding = global.binding.expect("No binding found");

        let typ = &module.types[global.ty];

        match &typ.inner {
            naga::TypeInner::Struct { members, .. } => {
                // We flatten structs one level

                for member in members {
                    params.push(map_struct_member(module, member, res_binding));
                }
            }
            naga::TypeInner::Sampler { .. } => {
                params.push(Parameter::Opaque {
                    base_type: OpaqueBaseType::Sampler,
                    name,
                    group: res_binding.group,
                    binding: res_binding.binding,
                });
            }
            naga::TypeInner::Image { .. } => {
                params.push(Parameter::Opaque {
                    base_type: OpaqueBaseType::Image,
                    name,
                    group: res_binding.group,
                    binding: res_binding.binding,
                });
            }
            other => unimplemented!("{other:?}"),
        }
    }

    params
}

/// Maps a struct member to a parameter type
fn map_struct_member(
    module: &naga::Module,
    member: &naga::StructMember,
    res_binding: naga::ResourceBinding,
) -> Parameter {
    let name = member.name.clone().expect("No member name");
    let offset = member.offset;
    let mut member_type = &module.types[member.ty].inner;
    let size = member_type.size(module.to_ctx());

    log::trace!("Base type of `{name}`:\n{member_type:#?}");

    let mut array_length = 1;

    if let naga::TypeInner::Array { base, size, .. } = member_type {
        member_type = &module.types[*base].inner;

        array_length = get_concrete_size(*size).expect("No concrete array size");
    }

    let (base_type, base_size) = match member_type {
        naga::TypeInner::Matrix {
            columns,
            rows,
            scalar,
        } => (
            base_type_from_matrix(*columns, *rows),
            u32::from(scalar.width),
        ),
        naga::TypeInner::Array { size, stride, .. } => {
            // As we've already removed one layer of array indirection, we now see this inner type as "complex"

            (
                BufferBaseType::Complex,
                *stride * get_concrete_size(*size).expect("No concrete array size"),
            )
        }
        naga::TypeInner::Scalar(scalar) => {
            (base_type_from_scalar(*scalar), u32::from(scalar.width))
        }
        naga::TypeInner::Vector { size, scalar } => (
            base_type_from_vector(*scalar, *size),
            u32::from(scalar.width),
        ),
        other => unimplemented!("{:#?}", other),
    };

    Parameter::BufferMember {
        name,
        group: res_binding.group,
        binding: res_binding.binding,
        base_type,
        base_size,
        array_length,
        offset,
        size,
    }
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
