struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct ColorUniform {
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> u_color: ColorUniform;

@vertex
fn vs_main(@location(0) position: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return u_color.color;
}
