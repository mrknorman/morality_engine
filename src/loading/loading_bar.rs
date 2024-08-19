use std::time::Duration;
use rand::Rng;

use crate::text::{TextFrames, TextRaw};

use bevy::{
    asset::{processor::LoadAndSaveSettings, LoadedUntypedAsset}, ecs::component::StorageType, prelude::*, sprite::Anchor
};

#[derive(Component)]
pub struct LoadingText;

#[derive(Component)]
pub struct ProgressIndicator;

pub struct LoadingBar{
    timer : Timer,
    index : usize
}

impl LoadingBar {

    fn create_text_bundle(text: &str, font_size: f32, position: Vec3) -> Text2dBundle {
        Text2dBundle {
            text: Text {
                sections: vec![
                    TextSection::new(text, TextStyle {
                        font_size,
                        ..default()
                    })
                ],
                justify: JustifyText::Center,
                ..default()
            },
            text_anchor: Anchor::CenterLeft,
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        }
    }
    
    fn create_sprite_bundle(size: Vec2, position: Vec3) -> SpriteBundle {
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        }
    }
}

#[derive(Bundle)]
pub struct LoadingBarBundle{
    bar : LoadingBar,
    messages : TextFrames,
    visibility : VisibilityBundle,
    transform : TransformBundle
}

impl LoadingBarBundle {
    pub fn new(messages : Vec<String>) -> Self {

        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

        let timer_duration : Duration = Duration::from_secs_f32(
            rng.gen_range(1.0..=2.0)
        );
        
        Self {
            bar : LoadingBar {
                timer : Timer::new(
                    timer_duration, 
                TimerMode::Once
                ),
                index : 0
            },
            messages : TextFrames::new(messages),
            visibility : VisibilityBundle::default(),
            transform : TransformBundle::from_transform(
                Transform::from_xyz(0.0, 0.0, 0.0))
        }
    }

    pub fn fill(
        time: Res<Time>,
        mut loading_query : Query<(&mut Children, &mut LoadingBar, &TextFrames)>,      
        mut sprite_query : Query<&mut Sprite, With<ProgressIndicator>>,
        mut text_query : Query<&mut Text, With<LoadingText>>,
    ) {

        let mut rng = rand::thread_rng();

        for (
            children, mut bar, frames
        ) in loading_query.iter_mut() {

            if bar.timer.tick(time.delta()).just_finished() && bar.index < (frames.frames.len() - 1) {
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
                            let bar_size_increase = rng.gen_range(0.0..=100.0);
                            custom_size.x = (custom_size.x + bar_size_increase).min(494.0);
                            loading_finished = custom_size.x >= 494.0;
                        }
                    }   
                    
                    if let Ok(mut text) = text_query.get_mut(child) {
                        if !frames.frames.is_empty() {
                            if !loading_finished {
                                let new_index = bar.index % frames.frames.len();
                                text.sections[0].value = frames.frames[new_index].clone();
                            } else {
                                text.sections[0].value = frames.frames[frames.frames.len() - 1].clone();
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

                let messages: Option<Vec<String>> = {
                    let mut entity_mut = world.entity_mut(entity);
                    entity_mut.get_mut::<TextFrames>()
                        .map(
                            |frames| 
                            frames.frames.clone()
                        )
                };
                
        
                // Step 2: Spawn child entities and collect their IDs
                let mut commands: Commands = world.commands();

                if let Some(messages) = messages {
                
                    commands.entity(entity).with_children(|parent| {
                        parent.spawn(
                            SpriteBundle { 
                                sprite : Sprite {
                                    custom_size : Some(Vec2 {
                                        x : 0.0,
                                        y : 0.0
                                    }),
                                    ..default()
                                },
                                ..default()
                            }
                        );
                        parent.spawn((
                            LoadingText,
                            Self::create_text_bundle(
                                messages[0].as_str(), 
                                12.0, 
                                Vec3::new(-250.0, 20.0, 0.0)
                            ),
                        ));
                            
                        let sprite_sizes = vec![
                            Vec2::new(500.0, 2.0),
                            Vec2::new(500.0, 2.0),
                            Vec2::new(2.0, 15.0),
                            Vec2::new(2.0, 15.0),
                        ];
                            
                        let sprite_positions = vec![
                            Vec3::new(0.0, -7.5, 0.0),
                            Vec3::new(0.0, 7.5, 0.0),
                            Vec3::new(-250.0, 0.0, 0.0),
                            Vec3::new(250.0, 0.0, 0.0),
                        ];
                            
                        for (size, position) in sprite_sizes.iter().zip(sprite_positions.iter()) {
                            parent.spawn(Self::create_sprite_bundle(*size, *position));
                        }
                            
                        parent.spawn((
                            ProgressIndicator,
                            SpriteBundle {
                                sprite: Sprite {
                                    custom_size: Some(Vec2::new(0.0, 10.0)),
                                    anchor: Anchor::CenterLeft,
                                    ..default()
                                },
                                transform: Transform::from_xyz(-247.0, 0.0, 0.0),
                                ..default()
                            },
                        ));
                    });
                }
            }
        );
    }
}