use std::vec;
use bevy::prelude::*;
use crate::{
    audio::{
        ContinuousAudio, 
        ContinuousAudioPallet, 
        OneShotAudio, 
        OneShotAudioPallet
    }, game_states::{
        GameState, MainState, StateVector, SubState
    }, interaction::{
        InputAction, InteractionPlugin
    }, text::TextButtonBundle,
    timing::{TimingPlugin, TimerPallet}
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
                spawn_delayed_children,
                LoadingBarBundle::fill,
            )
                .run_if(in_state(GameState::Loading)),
        );

        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }

        if !app.is_plugin_added::<TimingPlugin>() {
            app.add_plugins(TimingPlugin);
        }
    }
}

#[derive(Component)]
pub struct LoadingRoot;

pub fn setup_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {    
    commands.spawn((
        LoadingRoot,
        StateScoped(GameState::Loading),
        TimerPallet::new(
            vec![
                ("narration".to_string(), 2.0),
                ("button".to_string(), 5.0)
            ]
        ),
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
        ),
        OneShotAudioPallet::new(
            vec![
                (
                    OneShotAudio::new(
                        &asset_server, 
                        "./sounds/startup_beep.ogg",
                        true,
                        0.1
                    )
                )
            ]
        )
    )).with_children(
        |parent| {
            parent.spawn(LoadingBarBundle::new(
                "text/loading_messages/loading.json"
            ));
        }
    );
}

pub fn spawn_delayed_children(
    mut commands: Commands,
    loading_query: Query<(Entity, &TimerPallet), With<LoadingRoot>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, timers) in loading_query.iter() {
        // Handle narration timer
        if let Some(narration_timer) = timers.timers.get("narration") {
            if narration_timer.just_finished() {
                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn(OneShotAudioPallet::new(vec![
                        OneShotAudio::new(
                            &asset_server,
                            "./sounds/intro_lilly_elvenlabs.ogg",
                            false,
                            1.0,
                        ),
                    ]));
                });
            }
        } else {
            warn!("Entity {:?} is missing the 'narration' timer", entity);
        }

        // Handle button timer
        if let Some(button_timer) = timers.timers.get("button") {
            if button_timer.just_finished() {
                let button_translation = Vec3::new(0.0, -150.0, 0.0);
                let next_state_vector = StateVector::new(
                    Some(MainState::InGame),
                    Some(GameState::Dialogue),
                    Some(SubState::Intro),
                );

                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn(TextButtonBundle::new(
                        &asset_server,
                        vec![
                            InputAction::PlaySound(String::from("click")),
                            InputAction::ChangeState(next_state_vector),
                        ],
                        vec![KeyCode::Enter],
                        "[Click here or Press Enter to Continue]",
                        button_translation,
                    ));
                });
            }
        } else {
            warn!("Entity {:?} is missing the 'button' timer", entity);
        }
    }
}