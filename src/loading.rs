use bevy::{prelude::*, time::Timer, sprite::Anchor};
use std::{path::PathBuf, vec, time::Duration};
use rand::Rng;

use crate::audio::{BackgroundAudio, play_sound_once};
use crate::io_elements::spawn_text_button;
use crate::game_states::{MainState, GameState, SubState};
use crate::narration::Narration;

#[derive(Resource)]
pub struct LoadingData {
	narration_audio_entity : Entity,
    text_button_entity : Entity,
    loading_bar_entity : Entity,
}

impl LoadingData {

    pub fn despawn(
        &self,
        mut commands: Commands,
    ) {
        commands.entity(self.narration_audio_entity).despawn_recursive();
        commands.entity(self.text_button_entity).despawn_recursive();
        commands.entity(self.loading_bar_entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct LoadingBar{
    text_vector : Vec<String>,
    timer : Timer,
    index : usize
}

#[derive(Component)]
pub struct LoadingText;


impl LoadingBar {

    pub fn new(text_vector : Vec<String>, index : usize) -> LoadingBar {

        let mut rng = rand::thread_rng();

        let timer_duration_millis : u64 = rng.gen_range(100..=2000) as u64;
        
        LoadingBar {
            text_vector,
            timer : Timer::new(Duration::from_millis(
                timer_duration_millis
            ), TimerMode::Once),
            index
        }
    }

    pub fn spawn_loading_bar  (
        text_vector : Vec<String>,
        commands: &mut Commands
    ) -> Entity {
        commands.spawn(
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
        ).with_children( |parent| {

            parent.spawn((
                LoadingText,
                Text2dBundle {
                text : Text {
                    sections : vec!(
                        TextSection::new(text_vector[0].as_str(), TextStyle {
                            font_size : 12.0,
                            ..default()
                        })
                    ),
                    justify : JustifyText::Center, 
                    ..default()
                },
                text_anchor : Anchor::CenterLeft,
                transform : Transform::from_xyz(-250.0, 20.0 ,0.0),
                ..default()
            }));
            parent.spawn(SpriteBundle { 
                sprite : Sprite {
                    custom_size : Some(Vec2 {
                        x : 500.0,
                        y : 2.0
                    }),
                    ..default()
                },
                transform : Transform::from_xyz(0.0, -7.5 ,0.0),
                ..default()
            });
            parent.spawn(SpriteBundle { 
                sprite : Sprite {
                    custom_size : Some(Vec2 {
                        x : 500.0,
                        y : 2.0
                    }),
                    ..default()
                },
                transform : Transform::from_xyz(0.0, 7.5 ,0.0),
                ..default()
            });
            parent.spawn(SpriteBundle { 
                sprite : Sprite {
                    custom_size : Some(Vec2 {
                        x : 2.0,
                        y : 15.0
                    }),
                    ..default()
                },
                transform : Transform::from_xyz(-250.0, 0.0 ,0.0),
                ..default()
            });
            parent.spawn(SpriteBundle { 
                sprite : Sprite {
                    custom_size : Some(Vec2 {
                        x : 2.0,
                        y : 15.0
                    }),
                    ..default()
                },
                transform : Transform::from_xyz(250.0, 0.0, 0.0),
                ..default()
            });

            parent.spawn((
                LoadingBar::new(text_vector, 0),
                SpriteBundle {
                sprite : Sprite {
                    custom_size : Some(Vec2 {
                        x : 0.0,
                        y : 10.0
                    }),
                    anchor: Anchor::CenterLeft,
                    ..default()
                },
                transform : Transform::from_xyz(-247.0, 0.0 ,0.0),
                ..default()

            }));

        } ).id()
    }

    pub fn fill_loading_bar(
        time: Res<Time>,
        mut commands: Commands,
        mut loading_query : Query<(&Parent, &mut Sprite, &mut LoadingBar)>,
        mut text_query : Query<&mut Text, With<LoadingText>>
    ) {

        let mut rng = rand::thread_rng();

        let mut current_size : f32 = 0.0;

        for (parent, sprite, mut loading_bar) in loading_query.iter_mut() {

            if sprite.custom_size.is_some() {
                current_size = sprite.custom_size.unwrap().x;
            }            

            let bar_size_increase = rng.gen_range(0..=100) as f32;
            let mut new_size = current_size + bar_size_increase;
            new_size = if new_size < 494.0 {new_size} else {494.0};

            if loading_bar.timer.tick(time.delta()).just_finished() {

                if current_size < 494.0 {
            
                    commands.entity(**parent).with_children(|parent: &mut ChildBuilder<'_>| {
                        parent.spawn((
                            LoadingBar::new(loading_bar.text_vector.clone(), loading_bar.index + 1),
                            SpriteBundle {
                            sprite : Sprite {
                                custom_size : Some(Vec2 {
                                    x : new_size,
                                    y : 10.0
                                }),
                                anchor: Anchor::CenterLeft,
                                ..default()
                            },
                            transform : Transform::from_xyz(-247.0, 0.0 ,0.0),
                            ..default()

                        }));
                    });

                    let new_index = (loading_bar.index + 1) % loading_bar.text_vector.len();

                    for mut text in text_query.iter_mut() {
                        text.sections[0].value.clone_from(&loading_bar.text_vector[new_index]);
                    }

                } else {
                    for mut text in text_query.iter_mut() {
                        text.sections[0].value.clone_from(loading_bar.text_vector.last().unwrap());
                    }
                }
            }

        }

    }
}

pub fn setup_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    
    play_sound_once(
        "sounds/startup_beep.ogg", 
        &mut commands,
        &asset_server
    );

    let narration_audio_entity = commands.spawn((
        Narration {
            timer: Timer::from_seconds(2.0, TimerMode::Once)
        },
        AudioBundle {
			source: asset_server.load(PathBuf::from("./sounds/intro_lilly_elvenlabs.ogg")),
			settings : PlaybackSettings {
				paused : true,
				volume : bevy::audio::Volume::new(1.0),
				mode:  bevy::audio::PlaybackMode::Remove,
				..default()
			}
		}
    )).id();

    let text_button_entity = spawn_text_button(
		"[Click here or Press Enter to Begin]",
        Some(MainState::InGame),
		Some(GameState::Dialogue),
        Some(SubState::Intro),
        4.0,
        &mut commands
	); 

    let loading_bar_entity : Entity = LoadingBar::spawn_loading_bar(
        vec![
            String::from("INFO: Trol-E: Initilising boot up proceedure..."),
            String::from("INFO: Trol-E: Spooling Morality Cores..."),
            String::from("INFO: Trol-E: Thinking Therefore Existing..."),
            String::from("INFO: Trol-E: Performing Theseus Reconstitution..."),
            String::from("INFO: Trol-E: Pondering God's Existance..."),
            String::from("INFO: Trol-E: Evaluating Cavernous Shadow Conjecture..."),
            String::from("INFO: Trol-E: Determining Freedom Of Action Potential..."),
            String::from("INFO: Trol-E: Avoiding Regression Traps..."),
            String::from("INFO: Trol-E: Scrubbing Logical Paradoxes..."),
            String::from("INFO: Trol-E: Experiencing Qualia Integration Protocols..."),
            String::from("INFO: Trol-E: Solving the Fermi Paradox..."),
            String::from("INFO: Trol-E: Finalising Fomrulation of the First Cause..."),
            String::from("INFO: Trol-E: Complete.")
        ],
        &mut commands
    );

    let hum_audio = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/hum.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.5),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

    let office_audio = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/office.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.3),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

    let office_startup = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/startup.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.1),
			mode:  bevy::audio::PlaybackMode::Once,
			..default()
	}}).id();

    let background_audio: Vec<Entity> = vec![hum_audio, office_audio, office_startup];

    commands.insert_resource(BackgroundAudio{audio: background_audio});

    commands.insert_resource(LoadingData {
        narration_audio_entity,
        text_button_entity,
        loading_bar_entity
    });
}

pub fn cleanup_loading(
    mut commands: Commands,
    loading_data : Res<LoadingData>,
    background_audio : Res<BackgroundAudio>
    ) {

    for i in 0..background_audio.audio.len(){
        commands.entity(background_audio.audio[i]).despawn();
    }

    loading_data.despawn(commands);
}