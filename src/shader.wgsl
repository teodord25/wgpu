struct Uniforms {
    time: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@location(0) pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4(pos, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    let t = uniforms.time;
    return vec4(sin(t), cos(t), 0.5, 1.0);  // RGB color changes over time
}

