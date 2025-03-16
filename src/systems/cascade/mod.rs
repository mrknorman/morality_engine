use std::time::Duration;

use bevy::{
    ecs::component::{
        ComponentHooks, 
        StorageType
    }, 
    prelude::*, 
    sprite::Anchor, 
    text::LineBreak, 
    time::Stopwatch, 
    window::PrimaryWindow
};
use noise::NoiseFn;
use rand::Rng;

use crate::{
    data::rng::GlobalRng, 
    startup::render::MainCamera,
    systems::physics::Velocity
};

use super::{
    colors::{
        ColorAnchor, 
        OPTION_1_COLOR, 
        OPTION_2_COLOR,
        ColorExt
    }, 
    interaction::get_cursor_world_position, 
    resize::ResizeDebounce, 
    time::Dilation
};

pub struct CascadePlugin;
impl Plugin for CascadePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                Cascade::wrap,
                Cascade::update_visibility,
                Cascade::update_numbers,
                Cascade::enlarge,
                Cascade::resize,
                Ripple::spawn,
                Ripple::update_effect,
                Ripple::update,
            ))
            .insert_resource(ResizeDebounce::default())
            .register_required_components::<Cascade, Transform>()
            .register_required_components::<Cascade, Visibility>();
    }
}


#[derive(Clone, Component)]
pub struct Ripple {
    pub origin: Vec2,
    pub color : Color,
    pub stopwatch : Stopwatch
}

impl Ripple {

      // ----- Ripple Effect Constants -----
      const RIPPLE_SPEED: f32 = 100.0;   // Units per second
      const RIPPLE_WIDTH: f32 = 40.0;     // Thickness of the ripple band
      const RIPPLE_DURATION: f32 = 1.0;   // Total duration of the ripple effect in seconds
      const MAX_SCALE: f32 = 3.0;         // Maximum scale multiplier (when ripple effect is at its peak)
      
        fn spawn(
            mut commands: Commands,
            mut cascade_numbers: Query<Entity, With<Cascade>>,
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

            if let Some(color) = ripple_color {
                if let Some(cursor_pos) = get_cursor_world_position(&window, &camera_query) {
                    for entity in cascade_numbers.iter_mut() {
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn(Ripple {
                                origin: cursor_pos,
                                color,
                                stopwatch: Stopwatch::new(),
                            });
                        });
                    }
                }
            }
        }
      
        fn update(
            mut commands: Commands,
            time: Res<Time>,
            mut numbers: Query<(&Children, &mut Cascade)>,
            mut child_query: Query<(Entity, &mut Ripple)>,
            dilation: Res<Dilation>
        ) {
            let dt = time.delta_secs() * dilation.0;
            
            for (children, number) in numbers.iter_mut() {
                let speed = number.speed * dt;
                
                for &child in children.iter() {
                    let Ok((entity, mut ripple)) = child_query.get_mut(child) else { continue };
                    
                    ripple.stopwatch.tick(Duration::from_secs_f32(dt));
                    
                    if ripple.stopwatch.elapsed_secs() >= Self::RIPPLE_DURATION {
                        commands.entity(entity).despawn_recursive();
                        continue;
                    }
                    
                    ripple.origin.y -= speed;
                }
            }
        }
      
        fn update_effect(
            ripples: Query<(&Parent, &Ripple)>,
            mut numbers: Query<(&Parent, &GlobalTransform, &mut Transform, &mut TextColor, &ColorAnchor, &mut CascadeNumber)>,
        ) {
            // Group numbers by their parent
            for (number_parent, global_transform, mut transform, mut color, anchor, mut number) in numbers.iter_mut() {
                let mut ripple_effect: f32 = 0.0;
                let number_pos = global_transform.translation().truncate();
                let mut new_color = anchor.0.to_vec4();
                
                // Only process ripples that share the same parent as this number
                for (ripple_parent, ripple) in ripples.iter() {
                    // Skip if the ripple and number don't share the same parent

                    if ripple_parent.get().index() != number_parent.get().index() {
                        continue;
                    }
                    
                    let t = ripple.stopwatch.elapsed_secs();
                    let normalized_delta = (number_pos.distance(ripple.origin) - Self::RIPPLE_SPEED * t).abs() / Self::RIPPLE_WIDTH;
                    let effect = (1.0 - t / Self::RIPPLE_DURATION) 
                        * 0.5 * (1.0 + (normalized_delta * 0.5 * std::f32::consts::TAU).cos()) 
                        * f32::exp(-normalized_delta);

                    ripple_effect += effect;
                    new_color += (ripple.color.to_vec4() - anchor.0.to_vec4()) * effect;
                }
                
                if ripple_effect > 0.01 {
                    number.rippling = true;
                    color.0 = Color::LinearRgba(LinearRgba {
                        red: new_color.x,
                        green: new_color.y,
                        blue: new_color.z,
                        alpha: color.0.alpha(),
                    });
                    transform.scale = Vec3::splat(1.0 + (Self::MAX_SCALE - 1.0) * ripple_effect);
                } else {
                    number.rippling = false;
                }
            }
        }
}

// Main component for the cascading number grid
#[derive(Clone)]
pub struct Cascade {
    pub speed: f32,
    pub density: f32,
    pub visibility_speed: f64, // New field for controlling visibility fluctuation speed
    pub stopwatch : Stopwatch,
}

impl Default for Cascade {
    fn default() -> Self {
        Self {
            speed: 100.0,
            density: 2.0,
            visibility_speed: 0.02,
            stopwatch : Stopwatch::default(),
        }
    }
}

// Component for individual number characters
#[derive(Component)]
#[require(ColorAnchor)]
pub struct CascadeNumber {
    screen_height: f32,
    timer: Timer,
    noise_pos: [f64; 3],
    rippling : bool
}

// System to spawn the number grid when component is inserted
impl Component for Cascade {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(|mut world, entity, _| {
            // Get component data and validate resources exist
            let entity_ref = world.entity(entity);
            let Some(cascade) = entity_ref.get::<Cascade>().cloned() else { return };
            
            let Some(window) = world.iter_entities().find_map(|e| e.get::<Window>()).cloned() else {
                warn!("No window found! Cannot spawn cascade.");
                return;
            };

            if world.get_resource::<GlobalRng>().is_none() {
                warn!("GlobalRng not found! Cannot spawn cascade.");
                return;
            }

            const EDGE_OVERLAP: f32 = 50.0;
            let screen_width = window.width() / 2.0;
            let screen_height = window.height() / 2.0 + EDGE_OVERLAP;

            // Configure grid based on density
            let cell_size = 25.0 / cascade.density.sqrt();
            let grid_width = (screen_width * 2.0 / cell_size).ceil() as usize;
            let grid_height = (screen_height * 2.0 / cell_size).ceil() as usize;
            let total_cells = grid_width * grid_height;

            // Cache component values
            let text_font = entity_ref.get::<TextFont>().cloned().unwrap_or_else(|| 
                TextFont {
                    font_size: 10.0,
                    ..default()
                }
            );
            let text_color = entity_ref.get::<TextColor>().cloned().unwrap_or_default();

            // Pre-allocate vectors with capacity
            let mut cascade_configs = Vec::with_capacity(total_cells);
            let mut visibility_values = Vec::with_capacity(total_cells);
            let mut random_heights = Vec::with_capacity(total_cells);
            
            // Generate all configurations in one pass
            {
                let mut rng = world.resource_mut::<GlobalRng>();
                let perlin = rng.perlin;
                let noise_scale = 0.1;
                
                for col in 0..grid_width {
                    for row in 0..grid_height {
                        let x = (col as f32 * cell_size) - screen_width;
                        let y = (row as f32 * cell_size) - screen_height;
                        
                        let noise_pos = [
                            col as f64 * noise_scale,
                            row as f64 * noise_scale,
                            0.0,
                        ];
                        
                        let noise_value = perlin.get(noise_pos) + rng.uniform.gen_range(0.0..=1.0);
                        let is_visible = noise_value > 0.05;
                        let digit = if rng.uniform.gen_bool(0.5) { "1" } else { "0" };
                        
                        cascade_configs.push((
                            Vec3::new(x, y, 0.0),
                            digit.to_string(),
                            noise_pos,
                        ));

                        visibility_values.push(if is_visible { 
                            Visibility::Visible 
                        } else { 
                            Visibility::Hidden 
                        });
                        
                        random_heights.push(rng.uniform.gen_range(0.0..1.0));
                    }
                }
            }

            // Spawn all entities at once
            let mut commands = world.commands();
            commands.entity(entity).with_children(|parent| {
                for ((position, digit, noise_pos), random_height, visibility) in 
                    cascade_configs.into_iter()
                    .zip(random_heights)
                    .zip(visibility_values)
                    .map(|((a, b), c)| (a, b, c)) {
                    
                    parent.spawn((
                        CascadeNumber {
                            screen_height,
                            timer: Timer::from_seconds(random_height, TimerMode::Repeating),
                            noise_pos,
                            rippling : false
                        },
                        Velocity(Vec3::ZERO.with_y(-cascade.speed)),
                        Anchor::Center,
                        Text2d::new(digit),
                        text_font.clone(),
                        text_color,
                        TextLayout {
                            justify: JustifyText::Center,
                            linebreak: LineBreak::WordBoundary,
                        },
                        Transform { translation: position, ..default() },
                        visibility,
                    ));
                }
            });
        });
    }
}


impl Cascade {
    fn update_visibility(
        time: Res<Time>,
        dilation : Res<Dilation>,
        mut cascade_params: Query<&mut Cascade>,
        mut numbers: Query<(&CascadeNumber, &mut Visibility)>,
        global_rng: Res<GlobalRng>,
    ) {
    
        for mut cascade in cascade_params.iter_mut() {
    
            // Update the global timer
            cascade.stopwatch.tick(Duration::from_secs_f32(time.delta_secs() * dilation.0));
            
            let perlin = global_rng.perlin;
    
            // Update visibility for all cascade numbers
            for (number, mut visibility) in &mut numbers {
                // Create a time-varying noise coordinate
                let time_noise_pos = [
                    number.noise_pos[0],
                    number.noise_pos[1],
                    (cascade.stopwatch.elapsed_secs_f64() * 0.1 * cascade.visibility_speed) as f64,
                ];
                
                let noise_value = perlin.get(time_noise_pos) + (number.timer.duration().as_secs_f64() / 10.0); 
                let should_be_visible = noise_value > 0.5;
    
                if should_be_visible {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
    
    // System to wrap numbers when they fall off the bottom of the screen
    fn wrap(
        mut numbers: Query<(&CascadeNumber, &mut Transform)>,
    ) {
        for (number, mut transform) in &mut numbers {
            // If it's fallen below the screen, wrap it to the top
            if transform.translation.y < -number.screen_height {
                transform.translation.y = number.screen_height;
            }
        }
    }
    
    fn resize(
        mut commands: Commands,
        cascades: Query<(Entity, &Cascade)>,
        debounce: ResMut<ResizeDebounce>,
    ) {
    
        if debounce.timer.just_finished() {
            for (entity, cascade) in cascades.iter() {
                commands.entity(entity).despawn_descendants();
                commands.entity(entity).insert(cascade.clone());
            }
        }
    }
    
    // System to change numbers between 0 and 1
    pub fn update_numbers(
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
                
                // Update the text
                text.0 = new_digit.to_string();

                if number.rippling{
                    continue;
                }

                if rng.uniform.gen_range(0.0..1.0) > 0.9 {
                    if new_digit == "0" {
                        color.0 = OPTION_1_COLOR;
                    } else {
                        color.0 = OPTION_2_COLOR;
                    }
                } else {
                    color.0 = anchor.0;
                }
            }
        }
    }
    
    
    pub fn enlarge(
        mut query: Query<(&GlobalTransform, &mut Transform, &CascadeNumber)>,
        window: Single<&Window, With<PrimaryWindow>>,
        camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ) {
        let Some(cursor_position) = get_cursor_world_position(&window, &camera_query) else { return };
        
        // Configuration parameters
        let max_scale_factor = 2.0; // Maximum size multiplier (2x)
        let influence_radius = 150.0; // Distance in world units where scaling begins
        let min_distance = 10.0; // Distance below which maximum scaling is applied
        
        for (global_transform, mut transform, number) in &mut query {

            if number.rippling {
                continue;
            }

            // Get the position of the number in world space
            let number_position = global_transform.translation().truncate();
            
            // Calculate distance between cursor and number
            let distance = number_position.distance(cursor_position);
            
            // Only apply scaling if within influence radius
            if distance < influence_radius {
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
}