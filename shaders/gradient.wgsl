// time binding
// x = time in seconds
// y = time in seconds * 10
// z = sine of time
// w = cosine of time
@group(0) @binding(0) var<uniform> time: vec4<f32>;

// rect binding, like ratatui's Rect
// x = absolute x-position of rect
// y = absolute y-position of rect
// z = width of rect
// w = height of rect
@group(0) @binding(1) var<uniform> rect: vec4<f32>;

// fragment shader entry point
// we can use UV-coodinates as input in our fragment shader
// UV-coordinates are normalized (between 0 and 1) x/y-positions
// of the current pixel being processed
@fragment fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    // here we do some math to generate r, g, and b channels
    // trigonometry functions are useful as they return values
    // between -1 and 1 which we can re-map with (+ 1.0 * 0.5)
    // to the range 0 and 1.
    let r = sin(1.0 - uv.x + time.x) + 1.0 * 0.5;
    let g = cos(1.0 - uv.y + time.x) + 1.0 * 0.5;
    let b = 1.0 - r * g;

    // calculates the distance of the current pixel to the center (0.5, 0.5)
    // the closer we are the smaller the number.
    var d = distance(uv, vec2<f32>(0.5, 0.5));

    // let's turn that around by subtracting from 1.0
    d = 1.0 - d;

    // input our r, g, and b channels, with a constant alpha channel of 1.0
    // and darken the pixel the further it is from the center
    // we can exaggerate the darkening effect by applying it twice (d²)
    return vec4<f32>(r, g, b, 1.0) * d * d;
}

