use bevy::{
    anti_alias::{
        contrast_adaptive_sharpening::ContrastAdaptiveSharpening,
        fxaa::{Fxaa, Sensitivity},
    },
    asset::RenderAssetUsages,
    camera::{visibility::RenderLayers, RenderTarget},
    color::palettes::css::BLACK,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    image::ImageSampler,
    post_process::bloom::Bloom,
    post_process::effect_stack::ChromaticAberration,
    prelude::*,
    render::{
        render_resource::{
            AddressMode, AsBindGroup, Extent3d, FilterMode, SamplerDescriptor, ShaderType,
            TextureDimension, TextureFormat, TextureUsages,
        },
        view::{Hdr, Msaa},
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
    time::Real,
    window::{PrimaryWindow, WindowResized},
};
use noise::NoiseFn;
use rand_distr::{Distribution, Normal};

use crate::data::rng::GlobalRng;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum StartUpOrder {
    SetUpRenderTarget,
    ScanLineSetup,
}

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CrtSettings>()
            .init_resource::<ScreenShakeState>()
            .add_plugins(Material2dPlugin::<ScanLinesMaterial>::default())
            .add_systems(
                Startup,
                (
                    RenderTargetHandle::setup.in_set(StartUpOrder::SetUpRenderTarget),
                    ScanLinesMaterial::setup
                        .in_set(StartUpOrder::ScanLineSetup)
                        .after(StartUpOrder::SetUpRenderTarget),
                    setup_cameras.after(StartUpOrder::ScanLineSetup),
                ),
            )
            .add_systems(
                Update,
                (
                    update_screen_shake_state,
                    apply_camera_shake,
                    ScanLinesMaterial::update,
                    ScanLinesMaterial::update_scan_mesh,
                    RenderTargetHandle::update,
                ),
            );
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct CrtSettings {
    pub spacing: i32,
    pub thickness: i32,
    pub darkness: f32,
    pub curvature_strength: f32,
    pub static_strength: f32,
    pub jitter_strength: f32,
    pub aberration_strength: f32,
    pub phosphor_strength: f32,
    pub vignette_strength: f32,
    pub glow_strength: f32,
    // Legacy name kept for compatibility with existing menu/settings flow.
    // It now toggles CRT shader intensity, not the overall post-process pipeline.
    pub pipeline_enabled: bool,
}

impl Default for CrtSettings {
    fn default() -> Self {
        Self {
            spacing: 1,
            thickness: 1,
            darkness: 0.7,
            curvature_strength: 0.08,
            static_strength: 0.015,
            jitter_strength: 0.35,
            aberration_strength: 0.45,
            phosphor_strength: 0.7,
            vignette_strength: 0.8,
            glow_strength: 0.6,
            pipeline_enabled: true,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct ScreenShakeState {
    pub enabled: bool,
    pub target_intensity: f32,
    pub current_intensity: f32,
    pub max_pixels: f32,
    pub frequency_hz: f32,
    pub rise_per_second: f32,
    pub fall_per_second: f32,
}

impl ScreenShakeState {
    pub const DEFAULT_MAX_PIXELS: f32 = 4.8;
    pub const DEFAULT_FREQUENCY_HZ: f32 = 8.0;
    pub const DEFAULT_RISE_PER_SECOND: f32 = 5.0;
    pub const DEFAULT_FALL_PER_SECOND: f32 = 6.0;
}

impl Default for ScreenShakeState {
    fn default() -> Self {
        Self {
            enabled: true,
            target_intensity: 0.0,
            current_intensity: 0.0,
            max_pixels: Self::DEFAULT_MAX_PIXELS,
            frequency_hz: Self::DEFAULT_FREQUENCY_HZ,
            rise_per_second: Self::DEFAULT_RISE_PER_SECOND,
            fall_per_second: Self::DEFAULT_FALL_PER_SECOND,
        }
    }
}

fn update_screen_shake_state(time: Res<Time>, mut shake: ResMut<ScreenShakeState>) {
    let desired = if shake.enabled {
        shake.target_intensity.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let current = shake.current_intensity.clamp(0.0, 1.0);

    if (desired - current).abs() <= f32::EPSILON {
        shake.current_intensity = desired;
        return;
    }

    let delta = desired - current;
    let rate = if delta.is_sign_positive() {
        shake.rise_per_second.max(0.0)
    } else {
        shake.fall_per_second.max(0.0)
    };
    let step = rate * time.delta_secs();
    if step <= 0.0 {
        shake.current_intensity = current;
        return;
    }

    shake.current_intensity = if delta.is_sign_positive() {
        (current + step).min(desired)
    } else {
        (current - step).max(desired)
    };
}

const USE_POST_PROCESS: bool = true;

fn neutral_scanline_uniform(
    resolution: Vec2,
    real_time: f32,
    _screen_shake: ScreenShakeState,
) -> ScanLineUniform {
    ScanLineUniform {
        spacing: 0,
        thickness: 1,
        darkness: 0.0,
        curvature_strength: 0.0,
        static_strength: 0.0,
        jitter_strength: 0.0,
        aberration_strength: 0.0,
        phosphor_strength: 0.0,
        vignette_strength: 0.0,
        glow_strength: 0.0,
        // Camera shake is now transform-driven; shader shake stays disabled.
        shake_intensity: 0.0,
        shake_pixels: 0.0,
        shake_frequency: 0.0,
        resolution,
        real_time,
    }
}

fn scanline_uniform_from_settings(
    crt_settings: CrtSettings,
    _screen_shake: ScreenShakeState,
    resolution: Vec2,
    real_time: f32,
) -> ScanLineUniform {
    if !crt_settings.pipeline_enabled {
        // Keep the post-process pipeline active, but neutralize CRT effects so
        // color grading and bloom continue to run on the same camera path.
        return neutral_scanline_uniform(resolution, real_time, _screen_shake);
    }

    ScanLineUniform {
        spacing: crt_settings.spacing.max(0),
        thickness: crt_settings.thickness.max(1),
        darkness: crt_settings.darkness.clamp(0.0, 1.0),
        curvature_strength: crt_settings.curvature_strength.max(0.0),
        static_strength: crt_settings.static_strength.max(0.0),
        jitter_strength: crt_settings.jitter_strength.max(0.0),
        aberration_strength: crt_settings.aberration_strength.max(0.0),
        phosphor_strength: crt_settings.phosphor_strength.clamp(0.0, 1.0),
        vignette_strength: crt_settings.vignette_strength.clamp(0.0, 1.0),
        glow_strength: crt_settings.glow_strength.max(0.0),
        // Camera shake is now transform-driven; shader shake stays disabled.
        shake_intensity: 0.0,
        shake_pixels: 0.0,
        shake_frequency: 0.0,
        resolution,
        real_time,
    }
}

fn setup_cameras(
    mut commands: Commands,
    mut clear_color: ResMut<ClearColor>,
    render_target: Res<RenderTargetHandle>,
) {
    clear_color.0 = BLACK.into();

    if USE_POST_PROCESS {
        // Off‑screen camera: renders game geometry to the off‑screen texture.
        commands.spawn((
            Camera2d,
            OffscreenCamera,
            OffscreenCameraBaseTransform {
                translation: Vec3::ZERO,
                rotation: Quat::IDENTITY,
            },
            OffscreenCameraShakeRig::default(),
            RenderLayers::layer(0),
            Hdr,
            RenderTarget::Image(render_target.0.clone().into()),
            Camera::default(),
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
            DebandDither::Disabled,
            Fxaa {
                enabled: false,
                edge_threshold: Sensitivity::High,
                edge_threshold_min: Sensitivity::High,
            },
            ContrastAdaptiveSharpening {
                enabled: false,
                sharpening_strength: 0.6,
                denoise: false,
            },
            ChromaticAberration {
                intensity: 0.0,
                max_samples: 8,
                color_lut: None,
            },
            Msaa::Sample4,
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

#[derive(Component)]
struct OffscreenCameraBaseTransform {
    translation: Vec3,
    rotation: Quat,
}

#[derive(Component)]
struct OffscreenCameraShakeRig {
    offset: Vec2,
    velocity: Vec2,
    roll: f32,
    roll_velocity: f32,
    impact_timer: Timer,
}

impl Default for OffscreenCameraShakeRig {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            velocity: Vec2::ZERO,
            roll: 0.0,
            roll_velocity: 0.0,
            impact_timer: Timer::from_seconds(0.25, TimerMode::Repeating),
        }
    }
}

fn critically_damped_step_vec2(
    position: &mut Vec2,
    velocity: &mut Vec2,
    target: Vec2,
    stiffness: f32,
    damping: f32,
    delta_secs: f32,
) {
    let accel = (target - *position) * stiffness - *velocity * damping;
    *velocity += accel * delta_secs;
    *position += *velocity * delta_secs;
}

fn critically_damped_step_scalar(
    position: &mut f32,
    velocity: &mut f32,
    target: f32,
    stiffness: f32,
    damping: f32,
    delta_secs: f32,
) {
    let accel = (target - *position) * stiffness - *velocity * damping;
    *velocity += accel * delta_secs;
    *position += *velocity * delta_secs;
}

fn apply_camera_shake(
    time: Res<Time<Real>>,
    mut rng: ResMut<GlobalRng>,
    shake: Res<ScreenShakeState>,
    mut camera_query: Query<
        (
            &mut Transform,
            &OffscreenCameraBaseTransform,
            &mut OffscreenCameraShakeRig,
        ),
        With<OffscreenCamera>,
    >,
) {
    let intensity = if shake.enabled {
        shake.current_intensity.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let dt = time.delta_secs().clamp(0.0, 0.05);
    let t = time.elapsed_secs();

    for (mut transform, base, mut rig) in camera_query.iter_mut() {
        if intensity <= 0.0001 || shake.max_pixels <= 0.0 {
            let mut offset = rig.offset;
            let mut velocity = rig.velocity;
            critically_damped_step_vec2(&mut offset, &mut velocity, Vec2::ZERO, 220.0, 30.0, dt);
            rig.offset = offset;
            rig.velocity = velocity;

            let mut roll = rig.roll;
            let mut roll_velocity = rig.roll_velocity;
            critically_damped_step_scalar(&mut roll, &mut roll_velocity, 0.0, 180.0, 24.0, dt);
            rig.roll = roll;
            rig.roll_velocity = roll_velocity;
            transform.translation = base.translation + rig.offset.extend(0.0);
            transform.rotation = base.rotation * Quat::from_rotation_z(rig.roll);
            continue;
        }

        let cadence_hz =
            (shake.frequency_hz.max(0.5) * (0.4 + 0.8 * intensity.powf(0.7))).clamp(1.0, 16.0);
        let impact_interval = (1.0 / cadence_hz).clamp(0.06, 0.5);
        if (rig.impact_timer.duration().as_secs_f32() - impact_interval).abs() > f32::EPSILON {
            rig.impact_timer
                .set_duration(std::time::Duration::from_secs_f32(impact_interval));
        }
        rig.impact_timer.tick(time.delta());

        if rig.impact_timer.just_finished() {
            let impulse_sigma_x = (shake.max_pixels * intensity * 7.5).max(0.001) as f64;
            let impulse_sigma_y = (shake.max_pixels * intensity * 2.8).max(0.001) as f64;
            let normal_x = Normal::new(0.0, impulse_sigma_x)
                .expect("screen shake x impulse normal params must be valid");
            let normal_y = Normal::new(0.0, impulse_sigma_y)
                .expect("screen shake y impulse normal params must be valid");
            let roll_sigma = (0.0018 * intensity).max(0.0001) as f64;
            let normal_roll = Normal::new(0.0, roll_sigma)
                .expect("screen shake roll normal params must be valid");

            rig.velocity += Vec2::new(
                normal_x.sample(&mut rng.uniform) as f32,
                normal_y.sample(&mut rng.uniform) as f32,
            );
            rig.roll_velocity += normal_roll.sample(&mut rng.uniform) as f32;
        }

        // Low-frequency rumble + subtle vibration. Y amplitude stays lower than X
        // so the shake feels weighty rather than floaty.
        let rumble_x = rng.perlin.get([t as f64 * 1.15, 11.0, 0.0]) as f32;
        let rumble_y = rng.perlin.get([t as f64 * 0.95, 23.0, 0.0]) as f32;
        let vib_freq = 8.0 + 8.0 * intensity;
        let vibration = Vec2::new(
            (t * vib_freq * std::f32::consts::TAU).sin(),
            (t * vib_freq * 1.33 * std::f32::consts::TAU + 1.1).sin(),
        );
        let amp = shake.max_pixels * intensity;
        let target_offset = Vec2::new(
            rumble_x * (amp * 0.42) + vibration.x * (amp * 0.09),
            rumble_y * (amp * 0.18) + vibration.y * (amp * 0.04),
        );
        let target_roll = rumble_x * 0.0035 * intensity;

        let stiffness = 160.0 + 120.0 * intensity;
        let damping = 24.0 + 10.0 * intensity;
        let mut offset = rig.offset;
        let mut velocity = rig.velocity;
        critically_damped_step_vec2(
            &mut offset,
            &mut velocity,
            target_offset,
            stiffness,
            damping,
            dt,
        );
        rig.offset = offset;
        rig.velocity = velocity;

        let mut roll = rig.roll;
        let mut roll_velocity = rig.roll_velocity;
        critically_damped_step_scalar(&mut roll, &mut roll_velocity, target_roll, 140.0, 22.0, dt);
        rig.roll = roll;
        rig.roll_velocity = roll_velocity;

        let max_displacement = (shake.max_pixels * (0.45 + 0.75 * intensity)).max(0.1);
        if rig.offset.length() > max_displacement {
            rig.offset = rig.offset.normalize() * max_displacement;
            rig.velocity *= 0.8;
        }

        transform.translation = base.translation + rig.offset.extend(0.0);
        transform.rotation = base.rotation * Quat::from_rotation_z(rig.roll);
    }
}

#[derive(Resource, Default)]
struct RenderTargetHandle(pub Handle<Image>);

#[derive(Component)]
struct ScanMesh;

impl RenderTargetHandle {
    fn setup(mut commands: Commands, window: Single<&Window>, mut images: ResMut<Assets<Image>>) {
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

        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;

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
                if image.texture_descriptor.size.width != new_width
                    || image.texture_descriptor.size.height != new_height
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
        crt_settings: Res<CrtSettings>,
        screen_shake: Res<ScreenShakeState>,
    ) {
        // Get the primary window's resolution
        let window_resolution = Vec2::new(window.resolution.width(), window.resolution.height());

        // Create a fullscreen quad mesh.
        let mesh = meshes.add(Mesh::from(Rectangle::new(
            window.resolution.width(),
            window.resolution.height(),
        )));

        // Create our custom material with initial parameters.
        let material: Handle<ScanLinesMaterial> = materials.add(ScanLinesMaterial {
            source_texture: render_target.0.clone(),
            scan_line: scanline_uniform_from_settings(
                *crt_settings,
                *screen_shake,
                window_resolution,
                0.0,
            ),
        });

        // Spawn the quad with our custom material.
        commands.spawn((
            RenderLayers::layer(1), // Only sees the post-process quad
            Mesh2d(mesh),
            MeshMaterial2d(material),
            ScanMesh,
            Visibility::Visible,
        ));
    }

    /// Update the resolution uniform if the window size changes.
    fn update(
        window: Single<&Window, With<PrimaryWindow>>,
        time: Res<Time<Real>>,
        crt_settings: Res<CrtSettings>,
        screen_shake: Res<ScreenShakeState>,
        mut materials: ResMut<Assets<ScanLinesMaterial>>,
    ) {
        let window_resolution = Vec2::new(window.resolution.width(), window.resolution.height());
        for (_id, material) in materials.iter_mut() {
            material.scan_line = scanline_uniform_from_settings(
                *crt_settings,
                *screen_shake,
                window_resolution,
                time.elapsed_secs(),
            );
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
    pub curvature_strength: f32,
    pub static_strength: f32,
    pub jitter_strength: f32,
    pub aberration_strength: f32,
    pub phosphor_strength: f32,
    pub vignette_strength: f32,
    pub glow_strength: f32,
    pub shake_intensity: f32,
    pub shake_pixels: f32,
    pub shake_frequency: f32,
    pub resolution: Vec2,
    pub real_time: f32,
}

impl Material2d for ScanLinesMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/scan_lines.wgsl".into() // relative to the assets folder
    }
}

pub fn convert_to_hdr(base_image: &Image, boost: f32) -> Image {
    // Texture geometry -------------------------------------------------------
    let width = base_image.texture_descriptor.size.width;
    let height = base_image.texture_descriptor.size.height;

    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    // One RGBA32-float pixel = 4 channels × 4 bytes
    let mut hdr_data = Vec::with_capacity(
        width as usize * height as usize * 4 /*channels*/ * 4, /*bytes*/
    );

    // ------------------------------------------------------------------------
    let src_data = base_image
        .data
        .as_ref()
        .expect("Image has no CPU-side data"); // <-- unwrap the Option

    // Convert every 8-bit sRGB texel to linear f32, apply the boost factor
    for chunk in src_data.chunks(4) {
        // `chunks(4)` always yields a 4-byte slice for a valid RGBA8 texture
        let [r, g, b, a] = <[u8; 4]>::try_from(chunk).unwrap();

        let r = (r as f32 / 255.0) * boost;
        let g = (g as f32 / 255.0) * boost;
        let b = (b as f32 / 255.0) * boost;
        let a = a as f32 / 255.0; // usually you leave alpha un-boosted

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
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..default()
        }
        .into(),
    );

    hdr_image
}
