use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d
};

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PulsingMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    #[uniform(1)]
    pub phase: f32
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for PulsingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/pulsing_material.wgsl".into()
    }
}