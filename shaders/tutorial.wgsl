@group(0) @binding(0) var<uniform> time: vec4<f32>;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let r = sin(1.0 - uv.x + time.x) * 0.5 + 1;
    let g = -sin(1.0 - uv.y + time.x) * 0.5 + 1;
    let b = 0.5;

    let position = vec2<f32>(
        0.5 * (sin(time.x * 2.1) * 0.5 + 1),
        0.5 * (cos(time.x * 1.2) * 0.5 + 1)
    );
    
    let d = 1.0 - distance(position, uv);
    let color = vec4<f32>(r, g, b, 1.0) * d;
    return color;
}
