@group(0) @binding(0) var<uniform> time: f32;

fn hash_33(p: vec3<f32>) -> vec3<f32> {
    let d = vec3<f32>(
        dot(p, vec3<f32>(127.1, 311.7, 74.7)),
        dot(p, vec3<f32>(269.5, 183.3, 246.1)),
        dot(p, vec3<f32>(113.5, 271.9, 124.6))
    );
    let s = sin(d);
    let f = fract(s * 43758.5453123);
    return f;
}

fn voronoi_3(p: vec3<f32>) -> f32 {
    let p_floor = floor(p);
    let p_fract = fract(p);

    var res = 100.0;
    for (var x: f32 = -1.0; x <= 1.0; x = x + 1.0) {
        for (var y: f32 = -1.0; y <= 1.0; y = y + 1.0) {
            for (var z: f32 = -1.0; z <= 1.0; z = z + 1.0) {
                let b = vec3<f32>(x, y, z);
                let r = vec3<f32>(b) - p_fract + hash_33(p_floor + b);
                let d = dot(r, r);

                if d < res {
                    res = d;
                }
            }
        }
    }
    return res;
}

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let t = voronoi_3(vec3<f32>(uv.x * 5.0, uv.y * 5.0, time));
    let color = mix(vec4<f32>(0.6, 0.8, 0.9, 1.0), vec4<f32>(0.1, 0.3, 0.4, 1.0), t);
    return color;
}
