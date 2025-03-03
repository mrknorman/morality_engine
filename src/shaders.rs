use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
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
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ScanLinesMaterial {
    // Texture and sampler are on bindings 0 and 1, respectively.
    #[texture(0)]
    #[sampler(1)]
    pub source_texture: Handle<Image>,
    // All uniform parameters are grouped into a single uniform block at binding 2.
    #[uniform(2)]
    pub scan_line: ScanLineUniform,
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
pub struct ScanLineUniform {
    pub spacing: f32,
    pub thickness: f32,
    pub darkness: f32,
    pub resolution: Vec2,
}

impl Material2d for ScanLinesMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/scan_lines.wgsl".into() // relative to the assets folder
    }
}

impl ScanLinesMaterial {
    pub fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<ScanLinesMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
        windows: Query<&Window>,
    ) {
        let test_texture = asset_server.load("textures/test.png");

        // Get the primary window's resolution
        let window = windows.single();
        let window_resolution = Vec2::new(window.resolution.width(), window.resolution.height());

        // Create a fullscreen quad mesh.
        let mesh = meshes.add(Mesh::from(Rectangle::new(window.resolution.width(), window.resolution.height())));

        // Create our custom material with initial parameters.
        let material = materials.add(ScanLinesMaterial {
            source_texture: test_texture,
            scan_line: ScanLineUniform {
                spacing: 2.0,
                thickness: 2.0,
                darkness: 0.5,
                resolution: window_resolution,
            },
        });

        // Spawn the quad with our custom material.
        //commands.spawn((
        //    Mesh2d(mesh),
        //    MeshMaterial2d(material),
        //));
    }
}