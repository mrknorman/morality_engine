use bevy::{
    prelude::*, 
    time::Timer
};
use std::{
    collections::HashMap,
    path::PathBuf, 
    vec, 
};
use crate::{
    io_elements::{
        show_text_button, 
        text_button_interaction, 
        spawn_text_button, 
        check_if_enter_pressed
    },
    audio::{
        BackgroundAudio, 
        play_sound_once
    },
    game_states::{
        MainState, 
        GameState, 
        SubState
    },
    narration::{
        start_narration, 
        Narration
    }
};

mod loading_bar;

use loading_bar::LoadingBarBundle;

pub struct LoadingPlugin;
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Loading), setup_loading
        )
        .add_systems(
            Update,
            (
                start_narration,
                show_text_button,
                text_button_interaction,
                LoadingBarBundle::fill,
                check_if_enter_pressed,
            )
                .run_if(in_state(GameState::Loading)),
        )
        .add_systems(
            OnExit(GameState::Loading), 
            cleanup_loading
        );
    }
}

#[derive(Resource)]
pub struct LoadingData {
	narration_audio_entity : Entity,
    text_button_entity : Entity
}

impl LoadingData {

    pub fn despawn(
        &self,
        mut commands: Commands,
    ) {
        commands.entity(self.narration_audio_entity).despawn_recursive();
        commands.entity(self.text_button_entity).despawn_recursive();
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

    commands.spawn(LoadingBarBundle::new(
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
            String::from("INFO: Trol-E: Finalising Formulation of the First Cause..."),
            String::from("INFO: Trol-E: Complete.")
        ]
    ));

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

    let background_audio:HashMap<String, Entity>  = HashMap::from([
        ("hum".to_string(), hum_audio), 
        ("office".to_string(), office_audio), 
        ("startup".to_string(), office_startup)
    ]);

    commands.insert_resource(BackgroundAudio{audio: background_audio});

    commands.insert_resource(LoadingData {
        narration_audio_entity,
        text_button_entity
    });
}

pub fn cleanup_loading(
    mut commands: Commands,
    loading_data : Res<LoadingData>,
    background_audio : Res<BackgroundAudio>
    ) {

    let audio = background_audio.audio.clone();

    for (_, value) in audio {
        commands.entity(value).despawn();
    }

    loading_data.despawn(commands);
}