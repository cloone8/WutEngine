struct VSInput {
    @location(0) position: vec3<f32>,

//# if HAS_COLOR_MAP != 0
    @location(1) uv: vec2f,
//# endif
}

struct VSOutput {
    @builtin(position) position: vec4f,

    //# if HAS_COLOR_MAP != 0
    @location(0) uv: vec2f
    //# endif
}

@vertex fn vs(
    input: VSInput
) -> VSOutput {
    var vs_output: VSOutput;

    vs_output.position = instance_params.mvp * vec4f(input.position, 1.0);

    //# if HAS_COLOR_MAP != 0
    vs_output.uv = input.uv;
    //# endif

    return vs_output;
}

struct UserParams {
    base_color: vec4f
}

@group(WUTENGINE_MATERIAL_GROUP) @binding(0) var<uniform> params: UserParams;

//# if HAS_COLOR_MAP != 0
@group(WUTENGINE_MATERIAL_GROUP) @binding(1) var color_map_sampler: sampler;
@group(WUTENGINE_MATERIAL_GROUP) @binding(2) var color_map_texture: texture_2d<f32>;
//# endif

@fragment fn fs(fs_input: VSOutput) -> @location(0) vec4f {
    //# if HAS_COLOR_MAP != 0
    return textureSample(color_map_texture, color_map_sampler, fs_input.uv) * params.base_color;
    //# else
    return params.base_color;
    //# endif
}
