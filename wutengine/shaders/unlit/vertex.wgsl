struct wuteng_viewport_constants {
    view_mat: mat4x4<f32>,
    projection_mat: mat4x4<f32>,
    view_projection_mat: mat4x4<f32>

}

struct wuteng_instance_constants {
    model_mat: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> wuteng_vp_const_block: wuteng_viewport_constants; 

@group(0) @binding(1)
var<uniform> wuteng_instance_const_block: wuteng_instance_constants;

struct VertexOutput {
    @builtin(position) out_pos: vec4<f32>,
    @location(0) tex_coord: vec2<f32>
};

@vertex
fn vertex_main(
    @location(0) wuteng_position: vec3<f32>,
    @location(1) wuteng_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;

    out.out_pos = wuteng_vp_const_block.view_projection_mat * wuteng_instance_const_block.model_mat * vec4<f32>(wuteng_position, 1.0);
    out.tex_coord = wuteng_uv;

    return out;
}
