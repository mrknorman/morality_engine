use std::time::Duration;

use rand::Rng;
use serde::Deserialize;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld}, prelude::*, sprite::Anchor
};

use crate::{
    systems::{
        colors::{HIGHLIGHT_COLOR, PRIMARY_COLOR},
        time::Dilation
    }, 
    entities::{
        sprites::compound::HollowRectangle,
        text::TextFrames,
    },
    scenes::loading::content::LoadingBarMessages
};

pub struct LoadingBarPlugin;
impl Plugin for LoadingBarPlugin {
    fn build(&self, app: &mut App) {

        app
        .init_state::<LoadingSystemsActive>()
        .add_systems(Update, activate_systems)
        .add_systems(
            Update,
            (
                LoadingBar::fill,
            )
            .run_if(in_state(LoadingSystemsActive::True)),
        );
    }
}

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingSystemsActive {
    #[default]
    False,
    True
}

fn activate_systems(
    mut loading_state: ResMut<NextState<LoadingSystemsActive>>,
    loading_query: Query<&LoadingBar>
) {
    loading_state.set(if loading_query.is_empty() {
        LoadingSystemsActive::False
    } else {
        LoadingSystemsActive::True
    });
}

#[derive(Component)]
pub struct LoadingText;

#[derive(Component)]
pub struct ProgressIndicator;

#[derive(Component)]
#[require(Transform, Visibility)]
#[component(on_insert = LoadingBar::on_insert)]
pub struct LoadingBar {
    prefix: String,
    final_message: String,
    timer: Timer,
    index: usize,
    finished : bool
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
            finished: false
        }
    }

    pub fn load(loading_bar_messages : LoadingBarMessages) -> (LoadingBar, TextFrames) {
        let config = LoadingBarConfig::from_file(loading_bar_messages).expect(
            "Error reading Loading Bar configuration file!"
        );

        let mut rng: rand::prelude::ThreadRng = rand::rng();

        let timer_duration: Duration = Duration::from_secs_f32(
            rng.random_range(1.0..=2.0)
        );

        (
            LoadingBar::new(config.prefix, config.final_message, timer_duration),
            TextFrames::new(config.messages)
        )
    }

    pub fn fill(
        time: Res<Time>,
        dilation : Res<Dilation>,
        mut loading_query: Query<(&Children, &mut LoadingBar, &TextFrames)>,      
        mut sprite_query: Query<&mut Sprite, With<ProgressIndicator>>,
        mut text_query: Query<Entity, With<LoadingText>>,
        mut writer: Text2dWriter
    ) {
        let mut rng = rand::rng();

        for (children, mut bar, frames) in loading_query.iter_mut() {
            if bar.timer.tick(time.delta().mul_f32(dilation.0)).just_finished() {
                bar.index += 1;

                let secs = rng.random_range(0.5..=2.0) as f32;
                bar.timer = Timer::new(
                    Duration::from_secs_f32(secs), 
                    TimerMode::Once
                );

                // Update sprite (progress indicator)
                for child in children.iter() {

                    if !bar.finished {
                        if let Ok(mut sprite) = sprite_query.get_mut(child) {
                            if let Some(custom_size) = &mut sprite.custom_size {
                                let bar_size_increase = rng.random_range(5.0..=50.0);
                                custom_size.x = (custom_size.x + bar_size_increase).min(494.0);
                                bar.finished = custom_size.x >= 494.0;
                            }
                        }  
                    }
                        
                    if let Ok(entity) = text_query.get_mut(child) {
                        if !frames.frames.is_empty() {
                            if !bar.finished {
                                let new_index = bar.index % frames.frames.len();
                                *writer.text(entity, 1) = frames.frames[new_index].clone();
                            } else {
                                *writer.text(entity, 1) = bar.final_message.clone();
                            }
                        }
                    }
                }
            };
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {

        let (messages, prefix) = {
            let entity_ref = world.entity(entity);
            let messages: Option<Vec<String>> = entity_ref
                .get::<TextFrames>()
                .map(|frames: &TextFrames| frames.frames.clone());
            let prefix: Option<String> = entity_ref
                .get::<LoadingBar>()
                .map(|bar: &LoadingBar| bar.prefix.clone());
            (messages, prefix)
        };

        // Step 2: Spawn child entities and collect their IDs
        let mut commands: Commands = world.commands();

        if let Some(messages) = messages {
            commands.entity(entity).with_children(|parent| {
                // Spawn the loading bar components
                parent.spawn((            
                    Sprite{
                        custom_size: Some(Vec2::ZERO),
                        color : PRIMARY_COLOR,
                        ..default()
                    },
                    Transform::default()
                ));
                
                parent.spawn((
                    LoadingText,
                    Text2d::new(prefix.unwrap_or_default()),
                    TextColor(PRIMARY_COLOR),
                    TextFont{
                        font_size : 12.0,
                        ..default()
                    },
                    Transform::from_xyz(-250.0, 20.0, 0.0),
                    TextLayout{
                        justify: JustifyText::Center,
                        ..default()
                    },
                    Anchor::CenterLeft,
                )).with_children( |parent| {
                    parent.spawn((
                            TextSpan::new(messages[0].clone()),
                            TextColor(PRIMARY_COLOR),
                            TextFont{
                                font_size : 12.0,
                                ..default()
                            },
                        )
                    );
                }); 
                
                parent.spawn(
                    HollowRectangle{
                        dimensions : Vec2::new(500.0, 20.0),
                        color : PRIMARY_COLOR,
                        ..default()
                    }
                );

                parent.spawn((
                    ProgressIndicator,
                    Sprite {
                        color: HIGHLIGHT_COLOR,
                        custom_size: Some(Vec2::new(0.0, 14.0)),
                        anchor: Anchor::CenterLeft,
                        ..default()
                    },
                    Transform::from_translation(
                        Vec3::ZERO.with_x(-247.0)
                    )
                ));
            });
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
    fn from_file(loading_bar_messages : LoadingBarMessages) -> Result<Self, serde_json::Error> {
        serde_json::from_str(loading_bar_messages.content())
    }
}