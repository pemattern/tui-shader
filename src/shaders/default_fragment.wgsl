struct FragmentInput {
    time: f32,
}

@group(0) @binding(0) var<uniform> input: FragmentInput;

@fragment
fn magenta(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
