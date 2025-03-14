use std::{iter::zip, time::Duration};

use bevy::{
    ecs::component::{ComponentHooks, StorageType}, prelude::*, sprite::Anchor, text::LineBreak, time::Stopwatch, window::{PrimaryWindow, WindowResized}
};
use noise::NoiseFn;
use rand::Rng;

use crate::{
    data::rng::GlobalRng, startup::render::MainCamera, systems::physics::Velocity
};

use super::{colors::{ColorAnchor, OPTION_1_COLOR, OPTION_2_COLOR}, interaction::get_cursor_world_position, time::Dilation};

// Plugin for the cascading number system
pub struct CascadePlugin;

impl Plugin for CascadePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                wrap_cascade_numbers,
                handle_resize_cascade,
                change_cascade_numbers,
                update_cascade_visibility, // Added new system
                enlarge_if_close,
                apply_ripple_effect,
                spawn_ripple_on_click,
                update_ripple_origin,
                update_ripples
            ))
            .insert_resource(ResizeDebounce::default())
            .insert_resource(VisibilityTimer::default()) // Added new resource
            .insert_resource(ActiveRipples::default())
            .register_required_components::<CascadeNumbers, Transform>()
            .register_required_components::<CascadeNumbers, Visibility>();
    }
}

// Main component for the cascading number grid
#[derive(Clone)]
pub struct CascadeNumbers {
    pub speed: f32,
    pub density: f32,
    pub visibility_speed: f32, // New field for controlling visibility fluctuation speed
}

impl Default for CascadeNumbers {
    fn default() -> Self {
        Self {
            speed: 100.0,
            density: 2.0,
            visibility_speed: 0.2 // Default value for visibility change speed
        }
    }
}

// Component for individual number characters
#[derive(Component)]
#[require(ColorAnchor)]
pub struct CascadeNumber {
    screen_height: f32,
    timer: Timer,
    noise_pos: [f64; 3], // Store the initial noise position (now 3D to include time)
    excited : bool
}

// New resource to track time for visibility changes
#[derive(Resource)]
pub struct VisibilityTimer {
    pub time: f32,
}

impl Default for VisibilityTimer {
    fn default() -> Self {
        Self { time: 0.0 }
    }
}

// System to spawn the number grid when component is inserted
impl Component for CascadeNumbers {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _| {
            // Get required components and resources
            let entity_ref = world.entity(entity);
            let Some(cascade) = entity_ref.get::<CascadeNumbers>().cloned() else { return };
            
            // Find a window in the world
            let Some(window) = world.iter_entities().find_map(|e| e.get::<Window>()).cloned() else {
                warn!("No window found! Cannot spawn cascade.");
                return;
            };

            let screen_width = window.width() / 2.0;
            let screen_height = window.height() / 2.0 + 50.0;

            // Check if the RNG exists
            if world.get_resource::<GlobalRng>().is_none() {
                warn!("GlobalRng not found! Cannot spawn cascade.");
                return;
            }

            let text_font = entity_ref.get::<TextFont>().cloned();
            let text_color = entity_ref.get::<TextColor>().cloned();

            // Configure grid parameters - adjust cell_size based on density
            let base_cell_size = 25.0;
            let cell_size = base_cell_size / cascade.density.sqrt();
            let grid_width = (screen_width  * 2.0 / cell_size).ceil() as usize;
            let grid_height = (screen_height  * 2.0 / cell_size).ceil() as usize;

            // Pre-generate all the cascade number configurations
            let mut cascade_configs = Vec::new();
            let mut visibility_values = Vec::new();
            {
                let mut rng = world.resource_mut::<GlobalRng>();
                
                // Create Perlin noise generator with random seed
                let perlin = rng.perlin;
                let noise_scale = 0.1;
                
                // Generate grid of numbers
                for col in 0..grid_width {
                    for row in 0..grid_height {
                        // Calculate position
                        let x = (col as f32 * cell_size) - screen_width;
                        let y = (row as f32 * cell_size) - screen_height;
                        
                        // Store noise position for later updates
                        let noise_pos = [
                            col as f64 * noise_scale,
                            row as f64 * noise_scale,
                            0.0, // Initial time dimension
                        ];
                        
                        // Use Perlin noise to determine initial visibility
                        let noise_value = perlin.get(noise_pos) + rng.uniform.gen_range(0.0..=1.0);

                        let is_visible = noise_value > 0.05; // Adjust threshold to control sparsity
                        
                        // Random coin flip for 0 or 1
                        let digit = if rng.uniform.gen_bool(0.5) { "1" } else { "0" };
                        
                        cascade_configs.push((
                            Vec3::new(x, y, 0.0),
                            digit.to_string(),
                            noise_pos,
                        ));

                        if is_visible{
                            visibility_values.push(Visibility::Visible);
                        } else {
                            visibility_values.push(Visibility::Hidden);
                        }
                        
                    }
                }
            }

            let mut rng = world.resource_mut::<GlobalRng>();
            let random_heights: Vec<f32> = (0..cascade_configs.len())
                .map(|_| rng.uniform.gen_range(0.0..1.0))
                .collect();

            // Now we can use commands with a fresh mutable borrow
            let mut commands = world.commands();
            let mut parent_commands = commands.entity(entity);

            // Use the pre-generated configurations to spawn cascade numbers
            parent_commands.with_children(|parent| {
                for (((translation, digit, noise_pos), random_height), is_visible) in 
                    zip(zip(cascade_configs, random_heights), visibility_values) {
                    parent.spawn((
                        CascadeNumber {
                            screen_height,
                            timer: Timer::from_seconds(random_height, TimerMode::Repeating),
                            noise_pos,
                            excited : false
                        },
                        Velocity(Vec3::new(0.0, -cascade.speed, 0.0)), // Fall downward
                        Anchor::Center,
                        Text2d::new(digit),
                        text_font.clone().unwrap_or(
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                        ),
                        text_color.unwrap_or(
                            TextColor::default()
                        ),
                        TextLayout {
                            justify: JustifyText::Center,
                            linebreak: LineBreak::WordBoundary,
                        },
                        Transform {
                            translation,
                            ..default()
                        },
                        is_visible, // Set initial visibility
                    ));
                }
            });
        });
    }
}

// New system to update number visibility based on time-varying Perlin noise
fn update_cascade_visibility(
    mut timer: ResMut<VisibilityTimer>,
    time: Res<Time>,
    dilation : Res<Dilation>,
    cascade_params: Query<&CascadeNumbers>,
    mut numbers: Query<(&CascadeNumber, &mut Visibility)>,
    global_rng: Res<GlobalRng>,
) {
    // Get visibility speed from the first cascade parameters (if any)
    let visibility_speed = cascade_params
        .iter()
        .next()
        .map_or(0.2, |cascade| cascade.visibility_speed);
    
    // Update the global timer
    timer.time += time.delta_secs() * dilation.0 * visibility_speed;
    
    // Get perlin noise function
    let perlin = global_rng.perlin;

    // Update visibility for all cascade numbers
    for (number, mut visibility) in &mut numbers {
        // Create a time-varying noise coordinate
        let time_noise_pos = [
            number.noise_pos[0],
            number.noise_pos[1],
            timer.time as f64 * 0.1, // Scale time dimension
        ];
        
        // Get noise value and determine visibility
        let noise_value = perlin.get(time_noise_pos) + (number.timer.duration().as_secs_f64() / 10.0); 
        let should_be_visible = noise_value > 0.5;

        if should_be_visible {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

// System to wrap numbers when they fall off the bottom of the screen
fn wrap_cascade_numbers(
    mut numbers: Query<(&CascadeNumber, &mut Transform)>,
) {
    for (number, mut transform) in &mut numbers {
        // If it's fallen below the screen, wrap it to the top
        if transform.translation.y < -number.screen_height {
            transform.translation.y = number.screen_height;
        }
    }
}

// Debounce resource to throttle handling of resize events
#[derive(Resource)]
pub struct ResizeDebounce {
    pub timer: Timer,
    pub pending: bool,
}

impl Default for ResizeDebounce {
    fn default() -> Self {
        Self {
            // Wait 0.2 seconds after the last resize event before processing.
            timer: Timer::from_seconds(0.2, TimerMode::Once),
            pending: false,
        }
    }
}

fn handle_resize_cascade(
    mut commands: Commands,
    numbers: Query<Entity, With<CascadeNumber>>,
    cascades: Query<(Entity, &CascadeNumbers)>,
    resize_events: EventReader<WindowResized>,
    time: Res<Time>,
    mut debounce: ResMut<ResizeDebounce>,
) {
    // If any resize event is detected, mark as pending and reset timer.
    if !resize_events.is_empty() {
        debounce.pending = true;
        debounce.timer.reset();
    }

    // Tick the timer only if a resize is pending.
    if debounce.pending {
        debounce.timer.tick(time.delta());
        if debounce.timer.finished() {
            // Process the resize: remove current cascade numbers...
            for entity in numbers.iter() {
                commands.entity(entity).despawn_recursive();
            }
            // ...and reinsert the CascadeNumbers component to trigger the spawn hook.
            for (entity, cascade) in cascades.iter() {
                commands.entity(entity).insert(cascade.clone());
            }
            // Reset the pending flag.
            debounce.pending = false;
        }
    }
}

// System to change numbers between 0 and 1
pub fn change_cascade_numbers(
    time: Res<Time>,
    dilation : Res<Dilation>,
    mut rng: ResMut<GlobalRng>,
    mut query: Query<(&mut CascadeNumber, &mut Text2d, &mut TextColor, &ColorAnchor)>,
) {
    for (mut number, mut text, mut color, anchor) in &mut query {
        number.timer.tick(Duration::from_secs_f32(time.delta_secs() * dilation.0));        
        if number.timer.just_finished() {
            number.timer.set_duration(Duration::from_secs_f32(rng.uniform.gen_range(0.0..10.0)));
            // Flip the number
            let current_text = text.0.clone();
            let new_digit = if current_text == "0" { "1" } else { "0" };    

            if rng.uniform.gen_range(0.0..1.0) > 0.9 {
                if new_digit == "0" {
                    color.0 = OPTION_1_COLOR;
                } else {
                    color.0 = OPTION_2_COLOR;
                }
            } else {
                color.0 = anchor.0;
            }

            // Update the text
            text.0 = new_digit.to_string();
        }
    }
}


pub fn enlarge_if_close(
    mut query: Query<(&GlobalTransform, &mut Transform), With<CascadeNumber>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let Some(cursor_position) = get_cursor_world_position(&window, &camera_query) else { return };
    
    // Configuration parameters
    let max_scale_factor = 2.0; // Maximum size multiplier (2x)
    let influence_radius = 150.0; // Distance in world units where scaling begins
    let min_distance = 10.0; // Distance below which maximum scaling is applied
    
    for (global_transform, mut transform) in &mut query {
        // Get the position of the number in world space
        let number_position = global_transform.translation().truncate();
        
        // Calculate distance between cursor and number
        let distance = number_position.distance(cursor_position);
        
        // Only apply scaling if within influence radius
        if distance < influence_radius {
            // Calculate scaling factor based on distance
            // - At distance <= min_distance: scale_factor = max_scale_factor
            // - At distance >= influence_radius: scale_factor = 1.0
            // - In between: linear interpolation
            let scale_factor = if distance <= min_distance {
                max_scale_factor
            } else {
                let t = (influence_radius - distance) / (influence_radius - min_distance);
                1.0 + (max_scale_factor - 1.0) * t
            };
            
            // Apply the scaling (assuming original scale is 1.0)
            transform.scale = Vec3::splat(scale_factor);
        } else {
            // Reset scale when outside influence radius
            transform.scale = Vec3::ONE;
        }
    }
}

// ----- Ripple Effect Constants -----
const RIPPLE_SPEED: f32 = 100.0;   // Units per second
const RIPPLE_WIDTH: f32 = 40.0;     // Thickness of the ripple band
const RIPPLE_DURATION: f32 = 1.0;   // Total duration of the ripple effect in seconds
const MAX_SCALE: f32 = 3.0;         // Maximum scale multiplier (when ripple effect is at its peak)


// ----- Resource to track active ripples -----
#[derive(Resource, Default)]
pub struct ActiveRipples {
    pub ripples: Vec<Ripple>,
}

pub struct Ripple {
    pub origin: Vec2,
    pub color : Color,
    pub stopwatch : Stopwatch
}

// ----- System to spawn a ripple on mouse click -----
// This system listens for left mouse button clicks, converts the cursor
// position into world coordinates, and stores a new ripple in ActiveRipples.
fn spawn_ripple_on_click(
    mut active_ripples: ResMut<ActiveRipples>,
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // Determine if a left or right click occurred and select the corresponding color.
    let ripple_color = if input.just_pressed(MouseButton::Left) {
        Some(OPTION_1_COLOR)
    } else if input.just_pressed(MouseButton::Right) {
        Some(OPTION_2_COLOR)
    } else {
        None
    };

    // If a click was detected, compute the cursor position and spawn a ripple.
    if let Some(color) = ripple_color {
        if let Some(cursor_pos) = get_cursor_world_position(&window, &camera_query) {
            active_ripples.ripples.push(Ripple {
                origin: cursor_pos,
                color,
                stopwatch: Stopwatch::new(),
            });
        }
    }
}

fn update_ripples(
    time: Res<Time>,
    dilation: Res<Dilation>,
    mut active_ripples: ResMut<ActiveRipples>,
) {
    let dt = time.delta_secs() * dilation.0;
    for ripple in active_ripples.ripples.iter_mut() {
        ripple.stopwatch.tick(Duration::from_secs_f32(dt));
    }
    // Remove ripples that have expired.
    active_ripples
        .ripples
        .retain(|ripple| ripple.stopwatch.elapsed_secs() <= RIPPLE_DURATION);
}

// ----- System to apply the ripple effect to the cascade numbers -----
// For each cascade number, we compute the effect of every active ripple.
// The ripple effect is strongest when the number’s distance from the ripple’s
// origin is close to the current ripple radius. The effect decays both with
// distance (from the ripple front) and over time.

fn update_ripple_origin( 
    numbers: Query<(&CascadeNumbers)>,
    mut active_ripples: ResMut<ActiveRipples>,
    time : Res<Time>,
    dilation : Res<Dilation>
) {

    for number in numbers.iter() {
        let speed = number.speed * time.delta_secs() * dilation.0;
        for ripple in &mut active_ripples.ripples {
            ripple.origin -= Vec2::ZERO.with_y(speed);
        }
    }
}

fn apply_ripple_effect(
    mut active_ripples: ResMut<ActiveRipples>,
    mut query: Query<(&GlobalTransform, &mut Transform, &mut TextColor, &ColorAnchor), With<CascadeNumber>>,
) {
    for (global_transform, mut transform, mut color, anchor) in query.iter_mut() {
        let mut ripple_effect: f32 = 0.0;
        let number_pos = global_transform.translation().truncate();

        let mut new_color = anchor.0.to_vec4();
        for ripple in &mut active_ripples.ripples {
            let t = ripple.stopwatch.elapsed_secs();

            // Determine the current radius of the ripple
            let ripple_radius = RIPPLE_SPEED * t;
            // Compute distance from the number to the ripple origin
            let distance = number_pos.distance(ripple.origin);
            // How far is the number from the ripple's front?
            let delta = (distance - ripple_radius).abs();

            // Compute a multi-peak effect using a cosine function:
            let frequency = 0.5; // Number of peaks; adjust as needed
            let normalized_delta = delta / RIPPLE_WIDTH;
            let wave_effect = 0.5 * (1.0 + (normalized_delta * frequency * std::f32::consts::TAU).cos());
            let damping = f32::exp(-normalized_delta);
            let effect = (1.0 - t / RIPPLE_DURATION) * wave_effect * damping;

            ripple_effect += effect;

            new_color += (ripple.color.to_vec4() - anchor.0.to_vec4()) * effect;
        }

        if ripple_effect > 0.01 {
            color.0 = Color::LinearRgba(LinearRgba {
                red: new_color.x,
                green: new_color.y,
                blue: new_color.z,
                alpha: new_color.w,
            });

            // Apply the ripple effect as an additive scale on top of the base scale (1.0)
            transform.scale = Vec3::splat(1.0 + (MAX_SCALE - 1.0) * ripple_effect);
        } 
    }
}

trait ColorExt {
    fn to_vec4(self) -> Vec4;
}

impl ColorExt for Color {
    fn to_vec4(self) -> Vec4 {
        let color = self.to_linear();
        Vec4::new(color.red, color.green, color.blue, color.alpha)
    }
}