use std::{
    time::Duration,
    fs::File,
    io::BufReader,
    path::Path
};
use rand::Rng;
use serde::Deserialize;

use crate::{
    text::TextFrames,
    sprites::{
        SpriteFactory,
        SpriteBox
    }
};

use bevy::{
    prelude::*, 
    ecs::component::StorageType, 
    sprite::Anchor
};

#[derive(Component)]
pub struct LoadingText;

#[derive(Component)]
pub struct ProgressIndicator;


pub struct LoadingBar {
    prefix: String,
    final_message: String,
    timer: Timer,
    index: usize
}

impl LoadingBar {
    fn new(
        prefix: String, 
        final_message: String, 
        timer_duration: Duration
    ) -> Self {
        Self {
            prefix,
            final_message,
            timer: Timer::new(timer_duration, TimerMode::Once),
            index: 0,
        }
    }
}

#[derive(Deserialize)]
struct LoadingBarConfig {
    prefix: String,
    final_message: String,
    messages: Vec<String>,
}

impl LoadingBarConfig {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, serde_json::Error> {
        let file = File::open(path).expect("Could not open JSON file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
    }
}

#[derive(Bundle)]
pub struct LoadingBarBundle {
    bar: LoadingBar,
    messages: TextFrames,
    visibility: VisibilityBundle,
    transform: TransformBundle,
}

impl LoadingBarBundle {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let config = LoadingBarConfig::from_file(path).expect(
            "Error reading Loading Bar configuration file!"
        );

        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

        let timer_duration: Duration = Duration::from_secs_f32(
            rng.gen_range(1.0..=2.0)
        );

        Self {
            bar: LoadingBar::new(config.prefix, config.final_message, timer_duration),
            messages: TextFrames::new(config.messages),
            visibility: VisibilityBundle::default(),
            transform: TransformBundle::from_transform(
                Transform::from_xyz(0.0, 0.0, 0.0),
            ),
        }
    }

    pub fn fill(
        time: Res<Time>,
        mut loading_query: Query<(&mut Children, &mut LoadingBar, &TextFrames)>,      
        mut sprite_query: Query<&mut Sprite, With<ProgressIndicator>>,
        mut text_query: Query<&mut Text, With<LoadingText>>,
    ) {
        let mut rng = rand::thread_rng();

        for (children, mut bar, frames) in loading_query.iter_mut() {
            if bar.timer.tick(time.delta()).just_finished() {
                bar.index += 1;

                let secs = rng.gen_range(0.5..=2.0) as f32;
                bar.timer = Timer::new(
                    Duration::from_secs_f32(secs), 
                    TimerMode::Once
                );

                let mut loading_finished = false;
                // Update sprite (progress indicator)
                for &child in children.iter() {
                    if let Ok(mut sprite) = sprite_query.get_mut(child) {
                        if let Some(custom_size) = &mut sprite.custom_size {
                            let bar_size_increase = rng.gen_range(5.0..=50.0);
                            custom_size.x = (custom_size.x + bar_size_increase).min(494.0);
                            loading_finished = custom_size.x >= 494.0;
                        }
                    }   
                }
                
                for &child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        if !frames.frames.is_empty() {
                            if !loading_finished {
                                let new_index = bar.index % frames.frames.len();
                                text.sections[1].value = frames.frames[new_index].clone();
                            } else {
                                text.sections[1].value = bar.final_message.clone();
                            }
                        }
                    }
                }
            };
        }
    }
}

impl Component for LoadingBar {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {

                let (messages, prefix) = {
                    let entity_ref = world.entity(entity);
                    let messages = entity_ref
                        .get::<TextFrames>()
                        .map(|frames| frames.frames.clone());
                    let prefix = entity_ref
                        .get::<LoadingBar>()
                        .map(|bar| bar.prefix.clone());
                    (messages, prefix)
                };
        
                // Step 2: Spawn child entities and collect their IDs
                let mut commands: Commands = world.commands();

                if let Some(messages) = messages {
                    commands.entity(entity).with_children(|parent| {
                        // Spawn the loading bar components
                        parent.spawn(SpriteFactory::create_sprite_bundle(
                            Vec2::new(0.0, 0.0),
                            Vec3::new(0.0, 0.0, 0.0),
                        ));
                        
                        parent.spawn((
                            LoadingText,
                            SpriteFactory::create_text_bundle(
                                prefix.unwrap_or_default(),
                                messages[0].clone(), 
                                12.0, 
                                Vec3::new(-250.0, 20.0, 0.0)
                            ),
                        ));
                        
                        // Create and spawn the sprite box around the loading bar
                        let sprite_box = SpriteBox::create_sprite_box(
                            Vec3::new(0.0, 0.0, 0.0),
                            500.0,
                            20.0,
                        );
                        for sprite in sprite_box {
                            parent.spawn(sprite);
                        }

                        parent.spawn((
                            ProgressIndicator,
                            SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.0, 10.0)),
                                    anchor: Anchor::CenterLeft,
                                    ..default()
                                },
                                transform: Transform::from_xyz(
                                    -247.0, 0.0, 0.0
                                ),
                                ..default()
                            },
                        ));
                    });
                }
            }
        );
    }
}
