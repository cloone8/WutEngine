struct VSInput {
    @location(0) position: vec3<f32>,
    @builtin(vertex_index) vertex_index: u32
}

struct VSOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f
}


@vertex fn vs(
    input: VSInput
) -> VSOutput {
    // const fullscreen_triangle = array(
    //     vec2f(-1, -1),
    //     vec2f(3, -1),
    //     vec2f(-1, 3),
    // );

    const uvs = array(
        vec2f(0.0, 1.0),
        vec2f(2.0, 1.0),
        vec2f(0.0, -1.0),
    );

    // let pos = fullscreen_triangle[input.vertex_index];

    var vs_output: VSOutput;

    vs_output.position = instance_params.mvp * vec4f(input.position, 1.0);
    vs_output.uv = uvs[input.vertex_index];

    return vs_output;
}

struct UserParams {
    base_color: vec4f
}

@group(USER_PARAMS) @binding(0) var<uniform> params: UserParams;

//# if HAS_COLOR_MAP != 0
@group(USER_PARAMS) @binding(1) var<uniform> source_sampler: sampler;
@group(USER_PARAMS) @binding(2) var<uniform> source_texture: texture_2d<f32>;
//# endif

@fragment fn fs(fs_input: VSOutput) -> @location(0) vec4f {
    //# if HAS_COLOR_MAP != 0
    return textureSample(source_texture, source_sampler, fs_input.uv) * params.base_color * HAS_COLOR_MAP;
    //# else
    return params.base_color;
    //# endif
}
