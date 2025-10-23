use std::{collections::HashMap, time::Duration};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld}, 
    prelude::*, 
    sprite::Anchor, 
    text::LineBreak, 
    time::Stopwatch
};
use noise::NoiseFn;
use rand::Rng;    
use smallvec::SmallVec;

use crate::{
    data::rng::GlobalRng, 
    startup::cursor::CustomCursor,
    systems::physics::Velocity
};

use super::{
    colors::{
        ColorAnchor, 
        OPTION_1_COLOR, 
        OPTION_2_COLOR,
        ColorExt
    }, 
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
            .insert_resource(ResizeDebounce::default());
    }
}

#[derive(Clone, Component)]
pub struct Ripple {
    pub origin: Vec2,
    pub color : Color,
    pub stopwatch : Stopwatch
}

impl Ripple {
    const RIPPLE_SPEED: f32 = 100.0;
    const RIPPLE_WIDTH: f32 = 40.0;
    const RIPPLE_DURATION: f32 = 1.0;
    const MAX_SCALE: f32 = 3.0;

    fn spawn(
        mut commands: Commands,
        mut cascade_numbers: Query<Entity, With<Cascade>>,
        input: Res<ButtonInput<MouseButton>>,
        cursor: Res<CustomCursor>,
    ) {

        let Some(cursor_position) = cursor.position else { return };

        // Determine if a left or right click occurred and select the corresponding color.
        let color = if input.just_pressed(MouseButton::Left) {
            OPTION_1_COLOR
        } else if input.just_pressed(MouseButton::Right) {
            OPTION_2_COLOR
        } else {
            return;
        };

        for entity in cascade_numbers.iter_mut() {
            commands.entity(entity).with_children(|parent| {
                parent.spawn(Ripple {
                    origin: cursor_position,
                    color,
                    stopwatch: Stopwatch::new(),
                });
            });
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
            
            for child in children.iter() {
                let Ok((entity, mut ripple)) = child_query.get_mut(child) else { continue };
                
                ripple.stopwatch.tick(Duration::from_secs_f32(dt));
                
                if ripple.stopwatch.elapsed_secs() >= Self::RIPPLE_DURATION {
                    commands.entity(entity).despawn();
                    continue;
                }
                
                ripple.origin.y -= speed;
            }
        }
    }

    fn update_effect(
        ripples: Query<(&ChildOf, &Ripple), Changed<Ripple>>,
        mut numbers: Query<(&ChildOf, &GlobalTransform, &mut Transform,
                        &mut TextColor, &ColorAnchor, &mut CascadeNumber)>,
    ) {
        // -------- ① pre-group ripple refs ----------
        let mut by_parent: HashMap<Entity, SmallVec<[&Ripple; 4]>> = HashMap::default();
        for (parent, ripple) in ripples.iter() {
            by_parent.entry(parent.parent())
                    .or_default()
                    .push(ripple);
        }

        // -------- ② use tiny per-parent slice ------
        for (num_parent, gtr, mut tr, mut col, anchor, mut num) in numbers.iter_mut() {
            if let Some(ripples) = by_parent.get(&num_parent.parent()) {
                let mut effect = 0.0;
                let mut blended = anchor.0.to_vec4();
                let pos = gtr.translation().truncate();

                for r in ripples {
                    let t = r.stopwatch.elapsed_secs();
                    let delta = (pos.distance(r.origin) - Ripple::RIPPLE_SPEED * t).abs()
                                / Ripple::RIPPLE_WIDTH;
                    let e = (1.0 - t / Ripple::RIPPLE_DURATION)
                            * 0.5 * (1.0 + (delta * 0.5 * std::f32::consts::TAU).cos())
                            * (-delta).exp();
                    effect += e;
                    blended += (r.color.to_vec4() - anchor.0.to_vec4()) * e;
                }

                if effect > 0.01 {
                    num.rippling = true;
                    
                    col.0 = Color::LinearRgba(LinearRgba {
                        red: blended.x,
                        green: blended.y,
                        blue: blended.z,
                        alpha: col.0.alpha(),
                    });
                
                    tr.scale = Vec3::splat(1.0 + (Ripple::MAX_SCALE - 1.0) * effect);
                } else {
                    num.rippling = false;
                }
            }
        }
    }
}

// Main component for the cascading number grid
#[derive(Clone, Component)]
#[require(Transform, Visibility)]
#[component(on_insert = Cascade::on_insert)]
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
    state : bool,
    screen_height: f32,
    timer: Timer,
    noise_pos: [f64; 3],
    rippling : bool
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
            
            numbers.par_iter_mut().for_each(|(number, mut visibility)| {
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
            });
        }
    }

    fn wrap(mut numbers: Query<(&CascadeNumber, &mut Transform)>) {
        numbers.par_iter_mut().for_each(|(number, mut transform)| {
            if transform.translation.y < -number.screen_height {
                transform.translation.y = number.screen_height;
            }
        });
    }
    
    fn resize(
        mut commands: Commands,
        cascades: Query<(Entity, &Cascade)>,
        debounce: ResMut<ResizeDebounce>,
    ) {
        if debounce.timer.just_finished() {
            for (entity, cascade) in cascades.iter() {
                commands.entity(entity).despawn_related::<Children>();
                commands.entity(entity).insert(cascade.clone());
            }
        }
    }
    
    // System to change numbers between 0 and 1
    pub fn update_numbers(
        time: Res<Time>,
        dilation : Res<Dilation>,
        mut rng: ResMut<GlobalRng>,
        mut numbers: Query<(&mut CascadeNumber, &mut Text2d, &mut TextColor, &ColorAnchor)>,
    ) {

        let dt = Duration::from_secs_f32(time.delta_secs() * dilation.0);
        numbers.iter_mut().for_each(|(mut number, mut text, mut color, anchor)| {
            number.timer.tick(dt);        
            if number.timer.just_finished() {
                number.timer.set_duration(Duration::from_secs_f32(rng.uniform.random_range(0.0..10.0)));
                number.state = !number.state;                
                text.0 = if number.state {"1".to_string()} else { "0".to_string()};

                if !number.rippling{
                    if rng.uniform.random_range(0.0..1.0) > 0.9 {
                        if !number.state {
                            color.0 = OPTION_1_COLOR;
                        } else {
                            color.0 = OPTION_2_COLOR;
                        }
                    } else {
                        color.0 = anchor.0;
                    }
                }
            }
        });
    }
    
    pub fn enlarge(
        mut numbers: Query<(&GlobalTransform, &mut Transform, &CascadeNumber)>,
        cursor: Res<CustomCursor>
    ) {
        let Some(cursor_position) = cursor.position else { return };
        
        // Configuration parameters
        let max_scale_factor = 2.0; // Maximum size multiplier (2x)
        let influence_radius = 150.0; // Distance in world units where scaling begins
        let min_distance = 10.0; // Distance below which maximum scaling is applied
        
        numbers.par_iter_mut().for_each(|(global_transform,  mut transform, number)| {
            if !number.rippling {
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
                    transform.scale = Vec3::ONE;
                }
            }
        });
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {

        // Get component data and validate resources exist
        let entity_ref = world.entity(entity);
        let Some(cascade) = entity_ref.get::<Cascade>().cloned() else { return };
        
        let Some(window) = world
        .try_query::<&Window>()                      // get a query for all Window components
            .and_then(|mut q| q.iter(&world).next())     // iterate and take the first one
            .cloned()                                    // clone the Window so we can use it after borrow ends
        else {
            warn!("No window found! Cannot spawn.");
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
                    
                    let noise_value = perlin.get(noise_pos) + rng.uniform.random_range(0.0..=1.0);
                    let is_visible = noise_value > 0.05;
                    let digit = rng.uniform.random_bool(0.5);
                    
                    cascade_configs.push((
                        Vec3::new(x, y, 0.0),
                        digit,
                        noise_pos,
                    ));

                    visibility_values.push(if is_visible { 
                        Visibility::Visible 
                    } else { 
                        Visibility::Hidden 
                    });
                    
                    random_heights.push(rng.uniform.random_range(0.0..1.0));
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
                
                let text = if digit { "1" } else { "0" };
                parent.spawn((
                    CascadeNumber {
                        state : digit,
                        screen_height,
                        timer: Timer::from_seconds(random_height, TimerMode::Repeating),
                        noise_pos,
                        rippling : false
                    },
                    Velocity(Vec3::ZERO.with_y(-cascade.speed)),
                    Anchor::CENTER,
                    Text2d::new(text.to_string()),
                    text_font.clone(),
                    text_color,
                    TextLayout {
                        justify: Justify::Center,
                        linebreak: LineBreak::WordBoundary,
                    },
                    Transform { translation: position, ..default() },
                    visibility,
                ));
            }
        });
    }
}