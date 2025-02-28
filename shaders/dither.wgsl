struct FragmentInput {
    time: f32,
    padding: f32,
    resolution: vec2<u32>,
}

@group(0) @binding(0) var<uniform> input: FragmentInput;

fn permute_4_(x: vec4<f32>) -> vec4<f32> {
    return ((x * 34. + 1.) * x) % vec4<f32>(289.);
}

fn taylor_inv_sqrt_4_(r: vec4<f32>) -> vec4<f32> {
    return 1.79284291400159 - 0.85373472095314 * r;
}

fn simplex_3(v: vec3<f32>) -> f32 {
    let C = vec2(1. / 6., 1. / 3.);
    let D = vec4(0., 0.5, 1., 2.);

    // first corner
    var i = floor(v + dot(v, C.yyy));
    let x0 = v - i + dot(i, C.xxx);

    // other corners
    let g = step(x0.yzx, x0.xyz);
    let l = 1. - g;
    let i1 = min(g.xyz, l.zxy);
    let i2 = max(g.xyz, l.zxy);

    // x0 = x0 - 0. + 0. * C
    let x1 = x0 - i1 + 1. * C.xxx;
    let x2 = x0 - i2 + 2. * C.xxx;
    let x3 = x0 - 1. + 3. * C.xxx;

    // permutations
    i = i % vec3(289.);
    let p = permute_4_(permute_4_(permute_4_(
        i.z + vec4(0., i1.z, i2.z, 1.)) +
        i.y + vec4(0., i1.y, i2.y, 1.)) +
        i.x + vec4(0., i1.x, i2.x, 1.)
    );

    // gradients (NxN points uniformly over a square, mapped onto an octahedron)
    let n_ = 1. / 7.; // N=7
    let ns = n_ * D.wyz - D.xzx;

    let j = p - 49. * floor(p * ns.z * ns.z); // mod(p, N*N)

    let x_ = floor(j * ns.z);
    let y_ = floor(j - 7. * x_); // mod(j, N)

    let x = x_ * ns.x + ns.yyyy;
    let y = y_ * ns.x + ns.yyyy;
    let h = 1. - abs(x) - abs(y);

    let b0 = vec4(x.xy, y.xy);
    let b1 = vec4(x.zw, y.zw);

    let s0 = floor(b0) * 2. + 1.;
    let s1 = floor(b1) * 2. + 1.;
    let sh = -step(h, vec4(0.));

    let a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    let a1 = b1.xzyw + s1.xzyw * sh.zzww;

    var p0 = vec3(a0.xy, h.x);
    var p1 = vec3(a0.zw, h.y);
    var p2 = vec3(a1.xy, h.z);
    var p3 = vec3(a1.zw, h.w);

    // normalize gradients
    let norm = taylor_inv_sqrt_4_(vec4(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 = p0 * norm.x;
    p1 = p1 * norm.y;
    p2 = p2 * norm.z;
    p3 = p3 * norm.w;

    // mix final noise value
    var m = 0.6 - vec4(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3));
    m = max(m, vec4(0.));
    m *= m;
    return 42. * dot(m * m, vec4(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
}

const SOBEL_X: array<array<f32, 3>, 3> = array(
    array( 1.0, 0.0, -1.0 ),
    array( 2.0, 0.0, -2.0 ),
    array( 1.0, 0.0, -1.0 ),
);


const SOBEL_Y: array<array<f32, 3>, 3> = array(
    array(  1.0,  2.0,  1.0 ),
    array(  0.0,  0.0,  0.0 ),
    array( -1.0, -2.0, -1.0 ),
);

fn sobel(pixel_position: vec2<f32>) -> f32 {
    var sum_x = f32(0.0);
    var sum_y = f32(0.0);
    for (var x: i32 = -1; x <= 1; x++) {
        for (var y: i32 = -1; y <= 1; y ++) {
            let uv = (pixel_position + vec2<f32>(f32(x), f32(y))) / vec2<f32>(input.resolution);
            let sample = simplex_3(vec3<f32>(uv.x * 2.0, uv.y * 2.0, input.time * 0.35));
            sum_x = sum_x + sample * SOBEL_X[x + 1][y + 1];
            sum_y = sum_y + sample * SOBEL_Y[x + 1][y + 1];
        }
    }
    let result = sqrt(sum_x * sum_x + sum_y * sum_y);
    return 1.0 - result;
}

const BAYER_8X8: array<array<f32, 8>, 8> = array(
    array(0.0,      0.5,      0.125,    0.625,    0.03125,  0.53125,  0.15625,  0.65625 ),
    array(0.75,     0.25,     0.875,    0.375,    0.78125,  0.28125,  0.90625,  0.40625 ),
    array(0.1875,   0.6875,   0.0625,   0.5625,   0.21875,  0.71875,  0.09375,  0.59375 ),
    array(0.9375,   0.4375,   0.96875,  0.46875,  0.8125,   0.3125,   0.84375,  0.34375 ),
    array(0.046875, 0.546875, 0.171875, 0.671875, 0.015625, 0.515625, 0.171875, 0.671875),
    array(0.796875, 0.296875, 0.921875, 0.421875, 0.78125,  0.28125,  0.90625,  0.40625 ),
    array(0.234375, 0.734375, 0.109375, 0.609375, 0.203125, 0.703125, 0.078125, 0.578125),
    array(0.984375, 0.484375, 0.953125, 0.453125, 0.828125, 0.328125, 0.859375, 0.359375),
);

const margin: f32 = 0.15;
const dark_color = vec4<f32>(0.294, 0.278, 0.361, 1.0);
const light_color = vec4<f32>(0.843, 0.871, 0.863, 1.0);

@fragment
fn main(@location(0) uv: vec2<f32>, @builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let noise = simplex_3(vec3<f32>(uv.x * 2.0, uv.y * 2.0, input.time * 0.35));

    if uv.x < margin || uv.x > (1.0 - margin) || uv.y < (margin) || uv.y > (1.0 - margin) {
        let dither = BAYER_8X8[u32(position.x) % 8][u32(position.y) % 8];
        if (noise * 2.0) < dither {
            return dark_color;
        } else {
            return light_color;
        }
    } else {
        let sobel = sobel(position.xy);
        if sobel > 0.1 {
            return dark_color;
        } else {
            return light_color;
        }
    }
}
