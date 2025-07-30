struct wuteng_viewport_constants {
    view_mat: mat4x4<f32>,
    projection_mat: mat4x4<f32>,
    view_projection_mat: mat4x4<f32>
}

// struct wuteng_instance_constants {
//     model_mat: mat4x4<f32>
// }

@group(0) @binding(0)
var<uniform> viewport_constants: wuteng_viewport_constants; 

// @group(2) @binding(0)
// var<uniform> instance_constants: wuteng_instance_constants;

struct VertexOutput {
    @builtin(position) out_pos: vec4<f32>,
};

@vertex
fn vertex_main(
    @location(0) wuteng_position: vec3<f32>,
    // @location(1) wuteng_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;

    // out.out_pos = vec4<f32>(wuteng_position, 1.0);
    // out.out_pos = wuteng_vp_const_block.view_projection_mat * wuteng_instance_const_block.model_mat * vec4<f32>(wuteng_position, 1.0);
    out.out_pos = viewport_constants.view_projection_mat * vec4<f32>(wuteng_position, 1.0);
    // out.tex_coord = wuteng_uv;

    return out;
}

// @group(1) @binding(0)
// var<uniform> base_color: vec4<f32>;

//!!IF HAS_COLOR_MAP != 0
@group(1) @binding(1)
var color_map_tex: texture_2d<f32>;

@group(1) @binding(2)
var color_map: sampler;
//!!ENDIF

@fragment
fn fragment_main(
    // @location(0) tex_coord: vec2<f32>
    in: VertexOutput
) -> @location(0) vec4<f32> {
    //!!IF HAS_COLOR_MAP != 0
        return base_color * textureSample(color_map_tex, color_map, tex_coord);
    //!!ELSE
        // return base_color;
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    //!!ENDIF
}
