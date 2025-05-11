struct Uniforms {
    time: f32,
    center: vec2<f32>,
    zoom: f32,
    _padding: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let t = uniforms.time;

    let res = vec2(800.0, 400.0);
    var uv = (frag_coord.xy / res) * 2.0 - vec2(1.0, 1.0);
    uv.x *= res.x / res.y; // aspect ratio correction

    var c = uv / uniforms.zoom + uniforms.center;
    var z = vec2(0.0, 0.0);

    var iterations = 0;
    let max_iterations = 100;

    loop {
        if (length(z) > 2.0 || iterations >= max_iterations) {
            break;
        }

        let x = z.x * z.x - z.y * z.y + c.x;
        let y = 2.0 * z.x * z.y + c.y;
        z = vec2(x, y);

        iterations += 1;
    }

    let color_factor = f32(iterations) / f32(max_iterations);

    let r = 0.5 + 0.5 * sin(color_factor * 10.0);
    let g = 0.5 + 0.5 * sin(color_factor * 15.0);
    let b = 0.5 + 0.5 * sin(color_factor * 20.0);

    return vec4(r * color_factor, g * color_factor, b * color_factor, 1.0);
}
