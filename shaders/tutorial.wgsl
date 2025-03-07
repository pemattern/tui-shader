@group(0) @binding(0) var<uniform> time: vec4<f32>;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let r = sin(1.0 - uv.x + time.x);
    let g = cos(1.0 - uv.y + time.x);
    let b = 0.5;
    
    let d = 1.0 - distance(vec2<f32>(0.5), uv);
    let color = vec4<f32>(r, g, b, 1.0) * d;
    return color;
}
