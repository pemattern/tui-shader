@group(0) @binding(0) var<uniform> time: vec4<f32>;
@group(0) @binding(1) var<uniform> rect: vec4<u32>;

@fragment
fn green(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
