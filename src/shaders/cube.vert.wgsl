struct Camera { view_proj : mat4x4<f32> };
@group(0) @binding(0) var<uniform> camera : Camera;

struct Model { model : mat4x4<f32> };
@group(0) @binding(1) var<uniform> modelUBO : Model;

struct VSOut {
    @builtin(position) pos : vec4<f32>,
    @location(0) frag_pos : vec3<f32>,
    @location(1) normal   : vec3<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>,
           @location(1) normal: vec3<f32>) -> VSOut {
    let world_pos = modelUBO.model * vec4(position, 1.0);
    var out: VSOut;
    out.pos      = camera.view_proj * world_pos;
    out.frag_pos = world_pos.xyz;
    out.normal   = normalize((modelUBO.model * vec4(normal, 0.0)).xyz);
    return out;
}
