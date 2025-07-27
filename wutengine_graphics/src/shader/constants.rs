use glam::Mat4;

pub const VIEWPORT_CONSTANTS_BIND_GROUP: u32 = 0;

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ViewportConstants {
    pub view_mat: Mat4,
    pub projection_mat: Mat4,
    pub view_projection_mat: Mat4,
}
pub const INSTANCE_CONSTANTS_BIND_GROUP: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct InstanceConstants {
    pub model_mat: Mat4,
}

#[derive(Debug, Clone)]
pub struct RenderConstants {
    pub viewport: ViewportConstants,
    pub instance: InstanceConstants,
}
