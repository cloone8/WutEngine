//! Simple fullscreen blit shader

struct VSOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
}

@vertex fn vs(
    @builtin(vertex_index) vertex_index: u32
) -> VSOutput {
    const fullscreen_triangle = array(
        vec2f(-1, -1),
        vec2f(3, -1),
        vec2f(-1, 3),
    );

    const uvs = array(
        vec2f(0.0, 1.0),
        vec2f(2.0, 1.0),
        vec2f(0.0, -1.0),
    );

    let pos = fullscreen_triangle[vertex_index];

    var vs_output: VSOutput;

    vs_output.position = vec4f(pos, 0.0, 1.0);
    vs_output.uv = uvs[vertex_index];

    return vs_output;
}

@group(1) @binding(1) var source_sampler: sampler;
@group(1) @binding(2) var source_texture: texture_2d<f32>;

@fragment fn fs(fs_input: VSOutput) -> @location(0) vec4f {
    return textureSample(source_texture, source_sampler, fs_input.uv);
}
