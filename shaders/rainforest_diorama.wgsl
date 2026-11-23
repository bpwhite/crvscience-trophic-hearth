// TrophicHearth v0.1.0 — Rainforest Diorama PBR & Exponential Jungle Mist Shader (WGSL)
// Supports diurnal solar/lunar key light, canopy shadow dapple, wet-foliage specular sheen,
// nocturnal Mycena bioluminescence, and depth-layered atmospheric perspective.

struct EnvironmentUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir_and_intensity: vec4<f32>,
    sun_color: vec4<f32>,
    sky_ambient: vec4<f32>,
    ground_bounce: vec4<f32>,
    fog_color_and_density: vec4<f32>,
    params: vec4<f32>, // x: time_sec, y: wetness (0..1), z: night_glow (0..1), w: focus_stratum (-1 = all)
};

@group(0) @binding(0)
var<uniform> env: EnvironmentUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) material: vec4<f32>, // x: roughness, y: emissive, z: stratum_id, w: translucency
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) base_color: vec4<f32>,
    @location(3) material: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = env.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos = in.position;
    out.world_normal = normalize(in.normal);
    out.base_color = in.color;
    out.material = in.material;
    return out;
}

fn canopy_dapple(world_pos: vec3<f32>, t: f32) -> f32 {
    let canopy_height_mask = clamp((14.5 - world_pos.y) / 14.5, 0.0, 1.0);
    let wave_a = sin(world_pos.x * 0.85 + t * 0.35) * cos(world_pos.z * 0.75 - t * 0.22);
    let wave_b = sin((world_pos.x + world_pos.z) * 1.6 - t * 0.18);
    let dapple = 0.72 + 0.28 * clamp(wave_a * 0.65 + wave_b * 0.35, -1.0, 1.0);
    return mix(1.0, dapple, canopy_height_mask);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.world_normal);
    let v = normalize(env.camera_pos.xyz - in.world_pos);
    let l = normalize(env.sun_dir_and_intensity.xyz);
    let sun_int = env.sun_dir_and_intensity.w;

    // Hemisphere sky-to-soil ambient fill
    let hemi_t = clamp(n.y * 0.5 + 0.5, 0.0, 1.0);
    let ambient = mix(env.ground_bounce.rgb, env.sky_ambient.rgb, hemi_t);

    // Diffuse + foliage subsurface wrap lighting
    let wrap = in.material.w * 0.45;
    let n_dot_l = clamp((dot(n, l) + wrap) / (1.0 + wrap), 0.0, 1.0);
    let dapple = canopy_dapple(in.world_pos, env.params.x);

    // Wet-leaf rain specular highlight (Blinn-Phong modulated by rain wetness)
    let h = normalize(l + v);
    let roughness = clamp(in.material.x * (1.0 - 0.55 * env.params.y), 0.08, 1.0);
    let shininess = mix(96.0, 8.0, roughness);
    let spec_strength = mix(0.06, 0.42, env.params.y);
    let spec = pow(max(dot(n, h), 0.0), shininess) * spec_strength * sun_int;

    // Subtle rim light for depth separation (clamped against unit-vector dot > 1.0 NaN in pow)
    let rim = pow(clamp(1.0 - max(dot(n, v), 0.0), 0.0, 1.0), 3.2) * 0.18 * env.sky_ambient.rgb;

    // Nocturnal bioluminescence / eyeshine emission
    let biolum = in.base_color.rgb * in.material.y * (0.35 + 1.65 * env.params.z);

    var lit = in.base_color.rgb * (ambient + env.sun_color.rgb * sun_int * n_dot_l * dapple)
        + env.sun_color.rgb * spec
        + rim
        + biolum;

    // Trophic stratum spotlight dimming when a single trophic level is focused
    if (env.params.w >= -0.5) {
        let diff = abs(in.material.z - env.params.w);
        if (diff > 0.25) {
            let luma = dot(lit, vec3<f32>(0.2126, 0.7152, 0.0722));
            lit = mix(lit, vec3<f32>(luma) * 0.32, 0.68);
        }
    }

    // Exponential squared atmospheric jungle mist for deep Z layering
    let dist = length(env.camera_pos.xyz - in.world_pos);
    let fog_d = env.fog_color_and_density.w;
    let fog_factor = clamp(1.0 - exp(-(dist * fog_d) * (dist * fog_d)), 0.0, 1.0);
    let final_rgb = mix(lit, env.fog_color_and_density.rgb, fog_factor);

    return vec4<f32>(final_rgb, in.base_color.a);
}
