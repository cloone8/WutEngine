@group(1) @binding(0)
var<uniform> base_color: vec4<f32>;

//!!IF HAS_COLOR_MAP != 0
@group(1) @binding(1)
var color_map_tex: texture_2d<f32>;

@group(1) @binding(2)
var color_map: sampler;
//!!ENDIF

@fragment
fn fragment_main(
    @location(0) tex_coord: vec2<f32>
) -> @location(0) vec4<f32> {
    //!!IF HAS_COLOR_MAP != 0
        return base_color * textureSample(color_map_tex, color_map, tex_coord);
    //!!ELSE
        return base_color;
    //!!ENDIF
}
