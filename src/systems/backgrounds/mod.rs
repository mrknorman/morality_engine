use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::Rng;

use bevy::{
    ecs::{
        lifecycle::HookContext,
        system::SystemId, world::DeferredWorld
    }, prelude::*, sprite::Anchor, text::LineBreak, window::PrimaryWindow
};

use crate::{
    systems::physics::CameraVelocity, 
    entities::text::TextSprite,
    data::rng::GlobalRng
};

pub mod content;
use content::BackgroundTypes;

use super::resize::ResizeDebounce;

// States for background systems activation
#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackgroundSystemsActive {
    #[default]
    False,
    True
}

// Plugin for background implementation
pub struct BackgroundPlugin;
impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<BackgroundSystemsActive>()
            .add_systems(Update, toggle_background_systems)
            .add_systems(
                Update,
                (
                    BackgroundSprite::wrap_sprites,
                    Background::resize
                ).run_if(in_state(BackgroundSystemsActive::True))
            )
            .init_resource::<BackgroundSystems>();
    }
}

// Activates or deactivates systems based on whether background entities exist
fn toggle_background_systems(
    mut state: ResMut<NextState<BackgroundSystemsActive>>,
    backgrounds: Query<&Background>
) {
    let new_state = if backgrounds.is_empty() {
        BackgroundSystemsActive::False
    } else {
        BackgroundSystemsActive::True
    };
    
    state.set(new_state);
}

// Resource for background systems
#[derive(Resource)]
pub struct BackgroundSystems(pub HashMap<String, SystemId>);

impl FromWorld for BackgroundSystems {
    fn from_world(world: &mut World) -> Self {
        let mut systems = HashMap::new();
        
        systems.insert(
            "update_background_speeds".into(),
            world.register_system(Background::update_speeds)
        );
        
        BackgroundSystems(systems)
    }
}

// Background sprite configuration from JSON
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BackgroundSpriteType {
    name: String,
    frequency: f32,
    lods: Vec<String>
}

// Main background component
#[derive(Component, Clone)]
#[require(Transform, Visibility)]
#[component(on_insert = Background::on_insert)]
pub struct Background {
    sprites: Vec<BackgroundSpriteType>,
    pub density: f32,
    pub speed: f32
}

impl Background {
    /// Creates a new Background with the given type, density, and speed
    pub fn new(
        background_type: BackgroundTypes,
        density: f32,
        speed: f32,
    ) -> Self {
        let sprites: Vec<BackgroundSpriteType> = serde_json::from_str(background_type.content())
            .unwrap_or_else(|err| {
                panic!("Failed to parse background JSON: {}", err);
            });

        Self {
            sprites,
            density,
            speed,
        }
    }

    /// Handles window resize events by recreating all background sprites
    fn resize(
        mut commands: Commands,
        backgrounds: Query<(Entity, &Background)>,
        debounce: ResMut<ResizeDebounce>,
    ) {
        if debounce.timer.just_finished() {
            for (entity, background) in &backgrounds {
                commands.entity(entity).despawn_related::<Children>();
                commands.entity(entity).insert(background.clone());
            }
        }
    }

    /// Updates sprite velocities based on their vertical position
    pub fn update_speeds(
        window: Single<&Window, With<PrimaryWindow>>,
        backgrounds: Query<(&Children, &Background), Without<BackgroundSprite>>,
        mut sprites: Query<(&mut CameraVelocity, &Transform), Without<Background>>,
    ) {
        let screen_height = window.height();
    
        for (children, background) in &backgrounds {
            let parent_speed = background.speed;
            
            for &child_entity in children {
                if let Ok((mut velocity, transform)) = sprites.get_mut(child_entity) {
                    let distance_from_bottom = (screen_height - transform.translation.y).max(0.0);
                    let x_speed = distance_from_bottom * parent_speed;
                    velocity.0.x = x_speed;
                }
            }
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {

        let entity_ref = world.entity(entity);
        let Some(background) = entity_ref.get::<Background>().cloned() else { return };
        let color = entity_ref.get::<TextColor>().cloned();
        
        // Find a window in the world
        let Some(window) = world.iter_entities().find_map(|e| e.get::<Window>()).cloned() else {
            warn!("No window found! Cannot spawn background.");
            return;
        };

        let screen_width = window.width() / 2.0 + 100.0;
        let screen_height = window.height();

        // Closure to compute perspective scaling
        let perspective_scale = |y: f32| -> f32 {
            let t = (y + screen_height / 2.0) / screen_height; // normalized [0,1]
            1.0 - 0.5 * t // Scale from 1.0 (near) to 0.5 (far)
        };

        // First, check if the RNG exists
        if world.get_resource::<GlobalRng>().is_none() {
            warn!("GlobalRng not found! Cannot spawn background.");
            return;
        }

        // Generate all the random values we'll need upfront
        let mut sprite_configs = Vec::new();
        {
            let mut rng = world.resource_mut::<GlobalRng>();
            
            // Pre-generate all random positions for all sprites
            for sprite_type in &background.sprites {
                let num_lods = sprite_type.lods.len() as f32;
                let size_per_range = screen_height / num_lods;

                for (i, lod) in sprite_type.lods.iter().enumerate() {
                    let d0 = i as f32 * size_per_range;
                    let d1 = (i as f32 + 1.0) * size_per_range;
                    
                    let raw_density = background.density * sprite_type.frequency * (d1.powi(2) - d0.powi(2));
                    let density = raw_density.round() as i32;

                    for _ in 0..density {
                        let x_range = rng.uniform.random_range(-screen_width..screen_width + SPAWN_VARIANCE);
                        let y_in_range = rng.uniform.random_range(d0..d1);
                        let y_range = y_in_range - (screen_height / 2.0);
                        let random_offset = rng.uniform.random_range(screen_width..screen_width + SPAWN_VARIANCE);

                        let translation = Vec3::new(x_range, y_range, 0.0);
                        let scale_factor = perspective_scale(translation.y);
                        
                        sprite_configs.push((
                            translation,
                            scale_factor,
                            random_offset,
                            lod.clone(),
                            (screen_height / 2.0 - y_range).max(0.0) * background.speed * scale_factor
                        ));
                    }
                }
            }
        }

        // Now we can use commands with a fresh mutable borrow
        let mut commands = world.commands();
        let mut parent_commands = commands.entity(entity);

        // Use the pre-generated configurations to spawn sprites
        for (translation, scale_factor, random_offset, lod, speed) in sprite_configs {
            parent_commands.with_children(|parent| {
                let mut child_entity = parent.spawn((
                    BackgroundSprite {
                        screen_width,
                        random_offset,
                    },
                    CameraVelocity(Vec3::new(speed, 0.0, 0.0)),
                    Anchor::BOTTOM_CENTER,
                    Text2d::new(lod),
                    Transform {
                        translation,
                        scale: Vec3::splat(scale_factor),
                        ..Default::default()
                    },
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::WordBoundary,
                    },
                ));

                // Apply text color if available
                if let Some(ref text_color) = color {
                    child_entity.insert(text_color.clone());
                }
            });
        }

    }
}

// Configuration for sprite positioning and wrapping
const SPAWN_VARIANCE: f32 = 100.0;

#[derive(Component)]
#[require(TextSprite)]
pub struct BackgroundSprite {
    screen_width: f32,
    random_offset: f32
}

impl BackgroundSprite {
    /// Repositions sprites that have moved off-screen
    pub fn wrap_sprites(mut sprites: Query<(&BackgroundSprite, &mut Transform)>) {
        for (sprite, mut transform) in &mut sprites {
            if transform.translation.x <= -sprite.screen_width {
                transform.translation.x = sprite.random_offset;
            }
        }
    }
}