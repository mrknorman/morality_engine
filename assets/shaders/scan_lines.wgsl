#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var u_screenTex: texture_2d<f32>;
@group(2) @binding(1) var u_screenSampler: sampler;
@group(2) @binding(2) var<uniform> scanline: ScanlineSettings;

struct ScanlineSettings {
    spacing: i32,    // number of pixels for the dark gap
    thickness: i32,  // number of pixels for the bright line
    darkness: f32,
    resolution: vec2<f32>,
};

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(u_screenTex, u_screenSampler, mesh.uv);
    
    // Compute the integer y-coordinate of the current pixel.
    let pixelY: i32 = i32(mesh.uv.y * scanline.resolution.y);
    
    // Total period is the sum of the thickness (bright) and spacing (dark) regions.
    let period: i32 = scanline.thickness + scanline.spacing;
    
    // Use integer modulo to determine if the pixel is in the spacing (dark) region.
    if ((pixelY % period) < scanline.spacing) {
        color = vec4<f32>(color.rgb * (1.0 - scanline.darkness), color.a);
    }
    
    return color;
}