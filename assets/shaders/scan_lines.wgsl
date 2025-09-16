#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(0) var u_screenTex: texture_2d<f32>;
@group(2) @binding(1) var u_screenSampler: sampler;
@group(2) @binding(2) var<uniform> scanline: ScanlineSettings;

struct ScanlineSettings {
    spacing: i32,
    thickness: i32,
    darkness: f32,
    resolution: vec2<f32>,
};

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // ---------------------------
    // Normalize UV to [-1,1]
    var uv: vec2<f32> = mesh.uv * 2.0 - vec2<f32>(1.0, 1.0);

    // ---------------------------
    // Barrel distortion (curvature)
    let strength: f32 = 0.08;
    let r2: f32 = dot(uv, uv);
    uv = uv * (1.0 + strength * r2);

    // Back to [0,1]
    var warpedUV: vec2<f32> = uv * 0.5 + 0.5;

    // ---------------------------
    // Jitter / flicker (horizontal wobble)
    let jitter_amp: f32 = 1.0 / scanline.resolution.x * 0.07; 
    let jitter: f32 = sin(globals.time * 60.0 + warpedUV.y * 20.0) * jitter_amp;
    warpedUV.x += jitter;

    // Clamp to valid range
    if (warpedUV.x < 0.0 || warpedUV.x > 1.0 || warpedUV.y < 0.0 || warpedUV.y > 1.0) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    

    // ---------------------------
    // Chromatic aberration (RGB split)
    let aberration: f32 = 1.0 / scanline.resolution.x * 1.5;
    let r: f32 = textureSample(u_screenTex, u_screenSampler, warpedUV + vec2<f32>( aberration, 0.0)).r;
    let g: f32 = textureSample(u_screenTex, u_screenSampler, warpedUV).g;
    let b: f32 = textureSample(u_screenTex, u_screenSampler, warpedUV - vec2<f32>( aberration, 0.0)).b;
    var color: vec4<f32> = vec4<f32>(r, g, b, 1.0);

    // ---------------------------
    // Scanline effect
    let pixelY: i32 = i32(warpedUV.y * scanline.resolution.y);
    let period: i32 = scanline.thickness + scanline.spacing;
    if ((pixelY % period) < scanline.spacing) {
        color = vec4<f32>(color.rgb * (1.0 - scanline.darkness), color.a);
    }

    // ---------------------------
    // Phosphor mask (aperture grille effect)
    let pixelX: i32 = i32(warpedUV.x * scanline.resolution.x);
    let triad: i32 = pixelX % 3;
    if (triad == 0) {
        color.g *= 0.7; color.b *= 0.7;
    } else if (triad == 1) {
        color.r *= 0.7; color.b *= 0.7;
    } else {
        color.r *= 0.7; color.g *= 0.7;
    }

    // ---------------------------
    // Glow / Bloom boost
    let glow_boost: f32 = 1.25;
    let boosted_rgb: vec3<f32> = pow(color.rgb, vec3<f32>(1.0 / glow_boost));
    color = vec4<f32>(boosted_rgb, color.a);

    // ---------------------------
    // Vignette (edge darkness)
    let vignette_strength: f32 = 0.8;
    let vignette_radius: f32 = 0.85;
    let dist: f32 = length(uv);
    let vignette: f32 = smoothstep(vignette_radius, 1.2, dist);

    let factor: f32 = 1.0 - vignette * vignette_strength;
    let new_rgb: vec3<f32> = color.rgb * factor;
    color = vec4<f32>(new_rgb, color.a);

    // Noise / Static grain (apply last)
    let noise_seed: vec2<f32> = warpedUV * globals.time;
    let noise_val: f32 = fract(sin(dot(noise_seed, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let noise_strength: f32 = 0.015;

    let noisy_rgb: vec3<f32> = color.rgb + (noise_val - 0.5) * noise_strength;
    color = vec4<f32>(noisy_rgb, color.a);

    return color;
}
