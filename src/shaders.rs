use bevy::{
    prelude::*,
    render::{render_resource::{AsBindGroup, ShaderRef, ShaderType}, view::RenderLayers},
    sprite::Material2d, window::PrimaryWindow
};

use crate::{colors::HIGHLIGHT_COLOR, RenderTargetHandle};

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

#[derive(Component)]
pub struct ScanMesh;

impl ScanLinesMaterial {
    pub fn setup(
        mut commands: Commands,
        render_target: Res<RenderTargetHandle>,
        mut materials: ResMut<Assets<ScanLinesMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
        windows: Query<&Window>,
    ) {
        // Get the primary window's resolution
        let window = windows.single();
        let window_resolution = Vec2::new(window.resolution.width(), window.resolution.height());

        // Create a fullscreen quad mesh.
        let mesh = meshes.add(Mesh::from(Rectangle::new(window.resolution.width(), window.resolution.height())));

        // Create our custom material with initial parameters.
        let material = materials.add(ScanLinesMaterial {
            source_texture: render_target.0.clone(),
            scan_line: ScanLineUniform {
                spacing: 1.0,
                thickness: 0.5,
                darkness: 0.5,
                resolution: window_resolution,
            },
        });

        // Spawn the quad with our custom material.
        commands.spawn((
            RenderLayers::layer(1), // Only sees the post-process quad
            Mesh2d(mesh),
            MeshMaterial2d(material),
            ScanMesh
        ));
    }

    /// Update the resolution uniform if the window size changes.
    pub fn update(
        windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
        mut materials: ResMut<Assets<ScanLinesMaterial>>,
    ) {
        if let Ok(window) = windows.get_single() {
            let window_resolution = Vec2::new(window.resolution.width(), window.resolution.height());
            for (_id, material) in materials.iter_mut() {
                material.scan_line.resolution = window_resolution;
            }
        }
    }
    
    pub fn update_scan_mesh(
        windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
        mut meshes: ResMut<Assets<Mesh>>,
        scan_mesh_query: Query<&Mesh2d, With<ScanMesh>>,
    ) {
        if let Ok(window) = windows.get_single() {
            let new_resolution = Vec2::new(window.resolution.width(), window.resolution.height());
        
            for mesh_handle in scan_mesh_query.iter() {
                if let Some(mesh) = meshes.get_mut(mesh_handle) {
                    // Rebuild the mesh as a fullscreen quad with the new dimensions.
                    let new_mesh = Mesh::from(Rectangle::new(new_resolution.x, new_resolution.y));
                    *mesh = new_mesh;
                }
            }
        }
    }

}
