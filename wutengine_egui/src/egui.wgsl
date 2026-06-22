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

@vertex
fn vs(
    input: VSInput
) -> VSOutput {
    var vs_output: VSOutput;

    vs_output.position = position_from_screen(input.position.xy);
    vs_output.uv = input.uv;
    vs_output.color = input.color;

    return vs_output;
}

//# if DITHERING != 0

// -----------------------------------------------
// Adapted from
// https://www.shadertoy.com/view/llVGzG
// Originally presented in:
// Jimenez 2014, "Next Generation Post-Processing in Call of Duty"
//
// A good overview can be found in
// https://blog.demofox.org/2022/01/01/interleaved-gradient-noise-a-different-kind-of-low-discrepancy-sequence/
// via https://github.com/rerun-io/rerun/
fn interleaved_gradient_noise(n: vec2<f32>) -> f32 {
    let f = 0.06711056 * n.x + 0.00583715 * n.y;
    return fract(52.9829189 * fract(f));
}

fn dither_interleaved(rgb: vec3<f32>, levels: f32, frag_coord: vec4<f32>) -> vec3<f32> {
    var noise = interleaved_gradient_noise(frag_coord.xy);
    // scale down the noise slightly to ensure flat colors aren't getting dithered
    noise = (noise - 0.5) * 0.95;
    return rgb + noise / (levels - 1.0);
}

//# endif

@fragment
fn fs(fs_input: VSOutput) -> @location(0) vec4f {
    let tex_srgb = textureSample(ui_texture, ui_texture_sampler, fs_input.uv);

    //# if DITHERING != 0
        let out_color = fs_input.color * tex_srgb;
        return vec4f(dither_interleaved(out_color.rgb, 256.0, fs_input.position), out_color.a);
    //# else
        return fs_input.color * tex_srgb;
    //# endif
}
