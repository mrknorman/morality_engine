use bevy::{
    asset::RenderAssetUsages, color::palettes::css::BLACK, core_pipeline::{
        bloom::Bloom,
        tonemapping::Tonemapping,
    }, prelude::*, render::{
        camera::RenderTarget, 
        render_resource::{
            encase::private::ShaderType, AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat, TextureUsages
        }, 
        view::RenderLayers
    }, sprite::Material2d, window::{
        PrimaryWindow, 
        WindowResized
    }
};


#[derive(Component)]
pub struct MainCamera;

const USE_POST_PROCESS: bool = true;

pub fn setup_cameras(
    mut commands: Commands,         
    mut clear_color: ResMut<ClearColor>,
    render_target: Res<RenderTargetHandle>
) {
    clear_color.0 = BLACK.into();

    if USE_POST_PROCESS {
        // Off‑screen camera: renders game geometry to the off‑screen texture.
        commands.spawn((
            Camera2d,
            OffscreenCamera,
            RenderLayers::layer(0),
            Camera {
                hdr: true,
                target: RenderTarget::Image(render_target.0.clone()),
                ..default()
            },
            //Tonemapping::TonyMcMapface,
            //Bloom::default(),
        ));

        // Main (window) camera: renders only the fullscreen quad (post‑processing) to the window.
        commands.spawn((
            Camera2d,
            MainCamera,
            RenderLayers::layer(1),
            Camera {
                hdr: true,
                ..default()
            },
            Tonemapping::TonyMcMapface,
            Bloom::default(),
        ));
    } else {
        // Default camera: render directly to the window without any post‑processing.
        commands.spawn((
            Camera2d,
            MainCamera,
            Camera {
                hdr: true,
                ..default()
            },
            Tonemapping::TonyMcMapface,
            Bloom::default(),
        ));
    }
}


#[derive(Component)]
pub struct OffscreenCamera;

pub fn update_render_target_size(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut images: ResMut<Assets<Image>>,
    mut resize_reader: EventReader<WindowResized>,
    render_target: ResMut<RenderTargetHandle>,
) {
    for _ in resize_reader.read() {
        if let Ok(window) = windows.get_single() {
            let new_width = window.resolution.width() as u32;
            let new_height = window.resolution.height() as u32;
            // Skip update if the window is minimized (zero dimensions)
            if new_width == 0 || new_height == 0 {
                continue;
            }
            // Get a mutable reference to the current image, if it exists:
            if let Some(image) = images.get_mut(&render_target.0) {
                // If the size doesn’t match, recreate the image
                if image.texture_descriptor.size.width != new_width ||
                   image.texture_descriptor.size.height != new_height
                {
                    let new_size = Extent3d {
                        width: new_width,
                        height: new_height,
                        depth_or_array_layers: 1,
                    };
                    let clear_color = vec![0u8; 512 * 512 * 8]; // opaque black
                    // Create a new image filled with the clear color.
                    let mut new_image = Image::new_fill(
                        new_size,
                        TextureDimension::D2,
                        &clear_color,
                        TextureFormat::Rgba32Float,
                        RenderAssetUsages::default(),
                    );
                    // Ensure it has the correct usage flags.
                    new_image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT;
                    // Replace the old image with the new one.
                    *image = new_image;
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct RenderTargetHandle(pub Handle<Image>);

impl Default for RenderTargetHandle {
    fn default() -> Self {
        RenderTargetHandle(Handle::default())
    }
}

pub fn setup_render_target(
    mut commands: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = windows.single();
    let width = window.resolution.width() as u32;
    let height = window.resolution.height() as u32;
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let clear_color = vec![0u8; 512 * 512 * 8];
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &clear_color,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);
    commands.insert_resource(RenderTargetHandle(image_handle));
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
        let material: Handle<ScanLinesMaterial> = materials.add(ScanLinesMaterial {
            source_texture: render_target.0.clone(),
            scan_line: ScanLineUniform {
                spacing: 1,
                thickness: 1,
                darkness: 0.7,
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
    pub spacing: i32,
    pub thickness: i32,
    pub darkness: f32,
    pub resolution: Vec2
}

impl Material2d for ScanLinesMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/scan_lines.wgsl".into() // relative to the assets folder
    }
}