#import bevy_sprite::mesh2d_vertex_output::VertexOutput

#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> phase: f32;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate alpha using a sine wave based on the phase
    let alpha = pow(0.5 * (1.0 + sin(globals.time + phase)), 5.0);
    return material_color * vec4<f32>(alpha, alpha, alpha, alpha);
}