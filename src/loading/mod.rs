use bevy::{
    prelude::*, 
    time::Timer
};
use std::{
    ops::Sub, path::PathBuf, vec 
};
use crate::{
    audio::{
        ContinuousAudioPallet,
        ContinuousAudio,
        BackgroundAudio, 
        play_sound_once
    },
    game_states::{
        MainState, 
        GameState, 
        SubState,
        StateVector
    },
    narration::{
        start_narration, 
        Narration
    },
    text::TextButtonBundle,
    interaction::{
        InteractionPlugin,
        InputAction
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
                LoadingBarBundle::fill,
            )
                .run_if(in_state(GameState::Loading)),
        );

        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
    }
}

pub fn setup_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {

    let button_translation = Vec3::new(0.0, -50.0, 0.0);

    let next_state_vector = StateVector::new(
        Some(MainState::InGame),
        Some(GameState::Dialogue),
        Some(SubState::Intro)
    );
    
    commands.spawn((
        StateScoped(MainState::Menu),
        TransformBundle::from_transform(
            Transform::from_xyz(0.0, 0.0, 0.0)
        ),
        VisibilityBundle::default(),
        ContinuousAudioPallet::new(
            vec![
                (
                    "hum".to_string(),
                    ContinuousAudio::new(
                        &asset_server, 
                        "./sounds/hum.ogg", 
                        0.05
                    ),
                ),
                (
                    "office".to_string(),
                    ContinuousAudio::new(
                        &asset_server, 
                        "./sounds/office.ogg", 
                        1.0
                    ),
                )
            ]
        )
    )).with_children(
        |parent| {
            parent.spawn(
                // Add wait timer.
                TextButtonBundle::new(
                    &asset_server, 
                    vec![
                        InputAction::PlaySound(String::from("click")),
                        InputAction::ChangeState(next_state_vector)
                    ],
                    vec![KeyCode::Enter],
                    "[Click here or Press Enter to Begin]",
                    button_translation
                )
            );
        }
    );

    play_sound_once(
        "sounds/startup_beep.ogg", 
        &mut commands,
        &asset_server
    );

    commands.spawn((
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
    ));

    commands.spawn(LoadingBarBundle::new(
        "INFO::Trol-E: ",
        vec![
            String::from("Initilising boot up proceedure..."),
            String::from("Spooling Morality Cores..."),
            String::from("Thinking Therefore Existing..."),
            String::from("Performing Theseus Reconstitution..."),
            String::from("Pondering God's Existance..."),
            String::from("Evaluating Cavernous Shadow Conjecture..."),
            String::from("Determining Freedom Of Action Potential..."),
            String::from("Avoiding Regression Traps..."),
            String::from("Scrubbing Logical Paradoxes..."),
            String::from("Experiencing Qualia Integration Protocols..."),
            String::from("Solving the Fermi Paradox..."),
            String::from("Finalising Formulation of the First Cause..."),
            String::from("Complete.")
        ]
    ));

    commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/startup.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.1),
			mode:  bevy::audio::PlaybackMode::Once,
			..default()
	}});
}