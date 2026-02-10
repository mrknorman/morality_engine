use bevy::{
    asset::RenderAssetUsages, camera::{
        RenderTarget, 
        visibility::RenderLayers
    }, color::palettes::css::BLACK, core_pipeline::tonemapping::Tonemapping, image::ImageSampler, post_process::bloom::Bloom, prelude::*, render::{
        render_resource::{
            AddressMode, AsBindGroup, Extent3d, FilterMode, SamplerDescriptor, ShaderType, TextureDimension, TextureFormat, TextureUsages
        }, view::Hdr, 
    }, shader::ShaderRef, sprite_render::{Material2d, Material2dPlugin}, window::{
        PrimaryWindow, 
        WindowResized
    }
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum StartUpOrder {
    SetUpRenderTarget,
    ScanLineSetup
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(Material2dPlugin::<ScanLinesMaterial>::default())
        .add_systems(Startup, (
            RenderTargetHandle::setup
                .in_set(StartUpOrder::SetUpRenderTarget),
            ScanLinesMaterial::setup
                .in_set(StartUpOrder::ScanLineSetup)
                .after(StartUpOrder::SetUpRenderTarget),
            setup_cameras
                .after(StartUpOrder::ScanLineSetup)
        )).add_systems(Update, (
            ScanLinesMaterial::update, 
            ScanLinesMaterial::update_scan_mesh,
            RenderTargetHandle::update,
        ));
    }
}

#[derive(Component)]
pub struct MainCamera;

const USE_POST_PROCESS: bool = true;

fn setup_cameras(
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
            Hdr,
            RenderTarget::Image(render_target.0.clone().into())
        ));

        // Main (window) camera: renders only the fullscreen quad (post‑processing) to the window.
        commands.spawn((
            Camera2d,
            MainCamera,
            RenderLayers::layer(1),
            Hdr,
            Camera::default(),
            Tonemapping::TonyMcMapface,
            Bloom::default(),
        ));
    } else {
        // Default camera: render directly to the window without any post‑processing.
        commands.spawn((
            Camera2d,
            MainCamera,
            Hdr,
            Camera::default(),
            Tonemapping::TonyMcMapface,
            Bloom::default(),
        ));
    }
}


#[derive(Component)]
pub struct OffscreenCamera;

#[derive(Resource)]
struct RenderTargetHandle(pub Handle<Image>);

impl Default for RenderTargetHandle {
    fn default() -> Self {
        RenderTargetHandle(Handle::default())
    }
}

#[derive(Component)]
struct ScanMesh;

impl RenderTargetHandle {
    fn setup(
        mut commands: Commands,
        window: Single<&Window>,
        mut images: ResMut<Assets<Image>>,
    ) {
        let width = window.resolution.width() as u32;
        let height = window.resolution.height() as u32;
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
    
        // Calculate the exact buffer size needed
        let pixel_size_bytes = 16; // Rgba32Float = 4 channels × 4 bytes
        let buffer_size = (width * height * pixel_size_bytes as u32) as usize;
        let clear_color = vec![0u8; buffer_size];
        
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

    fn update(
        window: Single<&Window, (With<PrimaryWindow>, Changed<Window>)>,
        mut images: ResMut<Assets<Image>>,
        mut resize_reader: MessageReader<WindowResized>,
        render_target: ResMut<RenderTargetHandle>,
    ) {
        for _ in resize_reader.read() {
            let new_width = window.resolution.width() as u32;
            let new_height = window.resolution.height() as u32;
            // Skip update if the window is minimized (zero dimensions)
            if new_width == 0 || new_height == 0 {
                continue;
            }
            // Get a mutable reference to the current image, if it exists:
            if let Some(image) = images.get_mut(&render_target.0) {
                // If the size doesn't match, recreate the image
                if image.texture_descriptor.size.width != new_width ||
                    image.texture_descriptor.size.height != new_height
                {
                    let new_size = Extent3d {
                        width: new_width,
                        height: new_height,
                        depth_or_array_layers: 1,
                    };
                    
                    // Calculate the exact buffer size needed based on the format and dimensions
                    // For Rgba32Float, each pixel needs 16 bytes (4 channels × 4 bytes per float)
                    let pixel_size_bytes = 16; // Rgba32Float = 4 channels × 4 bytes
                    let buffer_size = (new_width * new_height * pixel_size_bytes as u32) as usize;
                    let clear_color = vec![0u8; buffer_size];
                    
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

impl ScanLinesMaterial {
    fn setup(
        mut commands: Commands,
        render_target: Res<RenderTargetHandle>,
        mut materials: ResMut<Assets<ScanLinesMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
        window: Single<&Window>,
    ) {

        // Get the primary window's resolution
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
    fn update(
        window: Single<&Window, (With<PrimaryWindow>, Changed<Window>)>,
        mut materials: ResMut<Assets<ScanLinesMaterial>>,
    ) {        
        let window_resolution = Vec2::new(window.resolution.width(), window.resolution.height());
        for (_id, material) in materials.iter_mut() {
            material.scan_line.resolution = window_resolution;
        }
    }
    
    fn update_scan_mesh(
        window: Single<&Window, (With<PrimaryWindow>, Changed<Window>)>,
        mut meshes: ResMut<Assets<Mesh>>,
        scan_mesh_query: Query<&Mesh2d, With<ScanMesh>>,
    ) {
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


#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ScanLinesMaterial {
    // Texture and sampler are on bindings 0 and 1, respectively.
    #[texture(0)]
    #[sampler(1)]
    pub source_texture: Handle<Image>,
    // All uniform parameters are grouped into a single uniform block at binding 2.
    #[uniform(2)]
    pub scan_line: ScanLineUniform,
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
struct ScanLineUniform {
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

pub fn convert_to_hdr(base_image: &Image, boost: f32) -> Image {
    // Texture geometry -------------------------------------------------------
    let width  = base_image.texture_descriptor.size.width;
    let height = base_image.texture_descriptor.size.height;

    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    // One RGBA32-float pixel = 4 channels × 4 bytes
    let mut hdr_data =
        Vec::with_capacity(width as usize * height as usize * 4 /*channels*/ * 4 /*bytes*/);

    // ------------------------------------------------------------------------
    let src_data = base_image
        .data
        .as_ref()
        .expect("Image has no CPU-side data");     // <-- unwrap the Option

    // Convert every 8-bit sRGB texel to linear f32, apply the boost factor
    for chunk in src_data.chunks(4) {
        // `chunks(4)` always yields a 4-byte slice for a valid RGBA8 texture
        let [r, g, b, a] = <[u8; 4]>::try_from(chunk).unwrap();

        let r = (r as f32 / 255.0) * boost;
        let g = (g as f32 / 255.0) * boost;
        let b = (b as f32 / 255.0) * boost;
        let a =  a as f32 / 255.0; // usually you leave alpha un-boosted

        hdr_data.extend_from_slice(&r.to_le_bytes());
        hdr_data.extend_from_slice(&g.to_le_bytes());
        hdr_data.extend_from_slice(&b.to_le_bytes());
        hdr_data.extend_from_slice(&a.to_le_bytes());
    }

    // Build the HDR image ----------------------------------------------------
    let mut hdr_image = Image::new(
        size,
        base_image.texture_descriptor.dimension,
        hdr_data,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    );

    // Usage flags
    hdr_image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    // Sampler
    hdr_image.sampler = ImageSampler::Descriptor(
        SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter:     FilterMode::Linear,
            min_filter:     FilterMode::Linear,
            mipmap_filter:  FilterMode::Linear,
            ..default()
        }
        .into(),
    );

    hdr_image
}
