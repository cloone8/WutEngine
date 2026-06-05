struct UserParams {
    screen_size: vec2f
}

@group(WUTENGINE_MATERIAL_GROUP) @binding(0) var<uniform> params: UserParams;

@group(WUTENGINE_MATERIAL_GROUP) @binding(1) var ui_texture_sampler: sampler;
@group(WUTENGINE_MATERIAL_GROUP) @binding(2) var ui_texture: texture_2d<f32>;

struct VSInput {
    @location(0) position: vec3f,
    @location(1) uv: vec2f,
    @location(2) color: vec4f,
}

struct VSOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) color: vec4f
}

fn position_from_screen(screen_pos: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(
        2.0 * screen_pos.x / params.screen_size.x - 1.0,
        1.0 - 2.0 * screen_pos.y / params.screen_size.y,
        0.0,
        1.0,
    );
}

@vertex fn vs(
    input: VSInput
) -> VSOutput {
    var vs_output: VSOutput;

    vs_output.position = position_from_screen(input.position.xy);
    vs_output.uv = input.uv;
    vs_output.color = input.color;

    return vs_output;
}


@fragment fn fs(fs_input: VSOutput) -> @location(0) vec4f {
    let tex_srgb = textureSample(ui_texture, ui_texture_sampler, fs_input.uv);
    
    return fs_input.color * tex_srgb;
}
