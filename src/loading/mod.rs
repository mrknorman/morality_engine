use std::vec;
use bevy::prelude::*;
use crate::{
    audio::{
        ContinuousAudio, 
        ContinuousAudioBundle,
        ContinuousAudioPallet, 
        MusicAudio, 
        NarrationAudio, 
        OneShotAudio, 
        OneShotAudioBundle,
        OneShotAudioPallet
    }, game_states::{
        GameState, 
        MainState,
        StateVector, 
        DilemmaPhase
    }, interaction::{
        InputAction,
        InteractionPlugin, 
        Clickable
    }, io::IOPlugin, 
    text::{
        TextButtonBundle,
        TextFrames 
    }, timing::{
        TimerPallet, 
        TimingPlugin, 
        TimerConfig,
        TimerStartCondition
    }, common_ui::{
        NextButtonBundle,
        NextButtonConfig
    }
};

mod loading_bar;
use loading_bar::LoadingBarBundle;

pub struct LoadingScreenPlugin;
impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            OnEnter(
                GameState::Loading
            ), 
            setup_loading
        )
        .add_systems(
            Update,
            (
                spawn_delayed_children,
                update_button_text,
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
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
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
                (
                    "music".to_string(),
                    TimerConfig::new(
                        TimerStartCondition::Immediate, 
                        2.0,
                        None
                    )
                ),
                (
                    "narration".to_string(),
                    TimerConfig::new(
                        TimerStartCondition::Immediate, 
                        3.0,
                        None
                    )
                ),
                (
                    "button".to_string(),
                    TimerConfig::new(
                        TimerStartCondition::Immediate, 
                        5.0,
                        None
                    )
                ),
                (
                    "update_button".to_string(),
                    TimerConfig::new(
                        TimerStartCondition::OnNarrationEnd, 
                        0.5,
                        Some(20.0)
                    )
                )
            ]
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
        ContinuousAudioPallet::new(
            vec![
                (
                    "hum".to_string(),
                    ContinuousAudio::new(
                        &asset_server, 
                        "./sounds/hum.ogg", 
                        0.1,
                        false
                    ),
                ),
                (
                    "office".to_string(),
                    ContinuousAudio::new(
                        &asset_server, 
                        "./sounds/office.ogg", 
                        0.5,
                        false
                    ),
                )
            ]
        ),
        OneShotAudioPallet::new(
            vec![
                OneShotAudio::new(
                    &asset_server, 
                    "./sounds/startup_beep.ogg",
                    true,
                    1.0
                )
            ]
        ),
    )).with_children(
        |parent| {
            parent.spawn(LoadingBarBundle::new(
                "text/loading_messages/bar.json"
            ));
        }
    );
}

pub fn spawn_delayed_children(
    mut commands: Commands,
    loading_query: Query<(Entity, &TimerPallet), With<LoadingRoot>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>
) {

    for (entity, timers) in loading_query.iter() {
        // Handle narration timer
        if let Some(narration_timer) = timers.timers.get(
            "narration"
        ) {
            if narration_timer.just_finished() {
                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn((
                        NarrationAudio,
                        OneShotAudioBundle::new(
                            &asset_server,
                            "./sounds/intro_lilly_elvenlabs.ogg",
                            false,
                            1.0,
                        )
                    ));
                });
            }
        } else {
            warn!("Entity {:?} is missing the 'narration' timer", entity);
        }

        if let Some(music_timer) = timers.timers.get(
            "music"
        ) {
            if music_timer.just_finished() {
                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn((
                        MusicAudio,
                        ContinuousAudioBundle::new(
                            &asset_server,
                            "./music/a_friend.ogg",
                            0.2,
                            false
                        )
                    ));
                });
            }
        } else {
            warn!("Entity {:?} is missing the 'music' timer", entity);
        }

        // Handle button timer
        if let Some(button_timer) = timers.timers.get(
            "button"
        ) {
            
            if button_timer.just_finished() {
                let next_state_vector = StateVector::new(
                    Some(MainState::InGame),
                    Some(GameState::Dialogue),
                    Some(DilemmaPhase::Intro),
                );

                let button_messages = TextFrames::load(
                    "text/loading_messages/button.json"
                );

                if let Ok(button_messages) = button_messages {
                    commands.entity(entity).with_children(|parent| {

                        let first_message = button_messages.frames[0].clone();
                        parent.spawn((
                            NextButtonBundle::new(),
                            button_messages,
                            TextButtonBundle::new(
                                &asset_server,
                                vec![
                                    InputAction::PlaySound(String::from("click")),
                                    InputAction::ChangeState(next_state_vector),
                                ],
                                vec![KeyCode::Enter],
                                first_message,
                                NextButtonBundle::translation(&windows)
                            ),
                        )); // Capture the entity ID of the spawned child
                    });
                }
            }
        } else {
            warn!("Entity {:?} is missing the 'button' timer", entity);
        }
    }
}

fn update_button_text(
    next_state_button: Res<NextButtonConfig>,
    loading_query: Query<(&TimerPallet, &LoadingRoot)>,
    mut text_query: Query<(&mut Text, &TextFrames, &mut Clickable)>,
) {
    let Some(button_entity) = next_state_button.entity() else {
        return;
    };

    let (text, frames, clickable) = match text_query.get_mut(button_entity) {
        Ok(components) => components,
        Err(_) => return,
    };

    let (timers, _) = loading_query
        .iter()
        .next()
        .expect("Expected at least one entity with TimerPallet and LoadingRoot");

    if let Some(update_button_timer) = timers.timers.get("update_button") {
        if update_button_timer.just_finished() {
            let index = update_button_timer.times_finished() as usize;
            if let Some(message) = frames.frames.get(index) {
                TextButtonBundle::change_text(message, text, clickable);
            }
        }
    }
}