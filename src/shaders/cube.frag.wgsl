struct Light { dir : vec3<f32>, color: vec3<f32> };
@group(0) @binding(2) var<uniform> light : Light;

@fragment
fn fs_main(
    @location(0) frag_pos: vec3<f32>,
    @location(1) normal:   vec3<f32>
) -> @location(0) vec4<f32> {
    let N  = normalize(normal);
    let L  = normalize(-light.dir);
    let diff = max(dot(N, L), 0.0);
    let base_color = vec3(0.3, 0.7, 1.0);
    let color = base_color * diff * light.color;
    return vec4(color, 1.0);
}
