// Octahedral‑to‑cube continuous mapping fragment shader
// One square texture (octahedral layout) → seamless cube‑sphere planet

struct Light { dir : vec3<f32>, color : vec3<f32> };

@group(0) @binding(2) var<uniform> light            : Light;
@group(0) @binding(3) var texture_data    : texture_2d<f32>;
@group(0) @binding(4) var texture_sampler : sampler;

/// Encode a unit vector to octahedral UV in [0,1]².
fn encode_octahedral(n : vec3<f32>) -> vec2<f32> {
    // rescale so |x|+|y|+|z| = 1
    let abs_n   = abs(n);
    let inv_sum = 1.0 / (abs_n.x + abs_n.y + abs_n.z + 1e-6);
    var uv      = n.xy * inv_sum; // −1…1 square

    // fold the lower hemisphere
    if (n.z < 0.0) {
        uv = (1.0 - abs(vec2<f32>(uv.y, uv.x))) * sign(uv);
    }
    return uv * 0.5 + vec2<f32>(0.5); // → 0…1
}

struct FSIn {
    @location(0) frag_pos : vec3<f32>,  // world or model‑space position
    @location(1) normal   : vec3<f32>,
};

@fragment
fn fs_main(in : FSIn) -> @location(0) vec4<f32> {
    // basic lambert lighting
    let N    = normalize(in.normal);
    let L    = normalize(-light.dir);
    let diff = max(dot(N, L), 0.0);

    // octahedral UV from surface direction
    let uv   = encode_octahedral(normalize(in.frag_pos));
    let tex  = textureSample(texture_data, texture_sampler, uv);

    let ambient = 0.1;
    let lit     = tex.rgb * (ambient + diff * light.color);
    return vec4<f32>(lit, tex.a);
}
