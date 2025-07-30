use wutengine_math::Mat4;

pub const VIEWPORT_CONSTANTS_BIND_GROUP: u32 = 0;
pub const MATERIAL_PARAMETERS_BIND_GROUP: u32 = 1;
pub const INSTANCE_CONSTANTS_BIND_GROUP: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ViewportConstants {
    pub view_mat: Mat4,
    pub projection_mat: Mat4,
    pub vp_mat: Mat4,
}

impl ViewportConstants {
    pub const IDENTITY: Self = Self {
        view_mat: Mat4::IDENTITY,
        projection_mat: Mat4::IDENTITY,
        vp_mat: Mat4::IDENTITY,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InstanceConstants {
    pub model_mat: Mat4,
    pub mvp_mat: Mat4,
}

#[derive(Debug, Clone)]
pub struct RenderConstants {
    pub viewport: ViewportConstants,
    pub instance: InstanceConstants,
}
