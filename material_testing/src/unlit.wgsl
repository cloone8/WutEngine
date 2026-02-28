struct VSInput {
    @location(0) position: vec3<f32>,
}

struct VSOutput {
    @builtin(position) position: vec4f,
}

@vertex fn vs(
    input: VSInput
) -> VSOutput {
    var vs_output: VSOutput;

    vs_output.position = instance_params.mvp * vec4f(input.position, 1.0);

    return vs_output;
}

struct UserParams {
    base_color: vec4f
}

@group(WUTENGINE_MATERIAL_GROUP) @binding(0) var<uniform> params: UserParams;

//# if HAS_COLOR_MAP != 0
@group(WUTENGINE_MATERIAL_GROUP) @binding(1) var<uniform> source_sampler: sampler;
@group(WUTENGINE_MATERIAL_GROUP) @binding(2) var<uniform> source_texture: texture_2d<f32>;
//# endif

@fragment fn fs(fs_input: VSOutput) -> @location(0) vec4f {
    //# if HAS_COLOR_MAP != 0
    return textureSample(source_texture, source_sampler, fs_input.uv) * params.base_color * HAS_COLOR_MAP;
    //# else
    return params.base_color;
    //# endif
}
