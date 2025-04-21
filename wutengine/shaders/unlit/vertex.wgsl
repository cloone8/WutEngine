struct view_projection {
    view: mat4x4<f32>,
    projection: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> wuteng_vp: view_projection;

@group(0) @binding(1)
var<uniform> wuteng_model: mat4x4<f32>; 

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

    out.out_pos = wuteng_vp.projection * wuteng_vp.view * wuteng_model * vec4<f32>(wuteng_position, 1.0);
    out.tex_coord = wuteng_uv;

    return out;
}
