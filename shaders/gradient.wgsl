struct FragmentInput {
    time: f32,
}

@group(0) @binding(0) var<uniform> input: FragmentInput;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let x = sin(1.0 - uv.x + input.time);
    asd
    let y = cos(1.0 - uv.y + input.time);

    return vec4<f32>(x, 0.5, y, 1.0);
}

