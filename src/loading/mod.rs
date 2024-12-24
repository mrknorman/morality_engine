use std::vec;
use bevy::{
    prelude::*,
    audio::Volume
};
use crate::{
    audio::{
        continuous_audio, 
        one_shot_audio, 
        ContinuousAudioPallet, 
        MusicAudio,
        NarrationAudio,
        OneShotAudio, 
        OneShotAudioPallet,
        TransientAudioPallet,
        TransientAudio
    }, common_ui::{
        NextButton,
        NextButtonConfig
    }, game_states::{
        DilemmaPhase, GameState, MainState, StateVector
    }, interaction::{
        Clickable, InputAction, InteractionPlugin
    }, io::IOPlugin, text::{
        TextButton,
        TextFrames 
    }, timing::{
        TimerConfig, TimerPallet, TimerStartCondition, TimingPlugin
    }
};

mod loading_bar;
use loading_bar::{LoadingBar, LoadingBarPlugin};

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
            )
                .run_if(in_state(GameState::Loading)),
        );

        if !app.is_plugin_added::<LoadingBarPlugin>() {
            app.add_plugins(LoadingBarPlugin);
        }
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
                    AudioPlayer::<AudioSource>(asset_server.load(
                        "./sounds/hum.ogg"
                    )),
                    PlaybackSettings{
                        paused : false,
                        volume : Volume::new(0.1),
                        ..continuous_audio()
                    }
                ),
                (
                    "office".to_string(),
                    AudioPlayer::<AudioSource>(asset_server.load(
                        "./sounds/office.ogg"
                    )),
                    PlaybackSettings{
                        paused : false,
                        volume : Volume::new(0.5),
                        ..continuous_audio()
                    }
                ),
            ]
        ),
        OneShotAudioPallet::new(
            vec![
                OneShotAudio{
                    source: asset_server.load(
                            "./sounds/startup_beep.ogg"
                        ),
                    persistent : true,
                    volume : 1.0
                }
            ]
        ),
    )).with_children(
        |parent| {
            parent.spawn(LoadingBar::load(
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
                        AudioPlayer::<AudioSource>(asset_server.load(
                            "./sounds/intro_lilly_elvenlabs.ogg", 
                        )),
                        PlaybackSettings{
                            volume : Volume::new(1.0),
                            ..one_shot_audio()
                        }
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
                        AudioPlayer::<AudioSource>(asset_server.load(
                            "./music/a_friend.ogg"
                        )),
                        PlaybackSettings{
                            paused : false,
                            volume : Volume::new(0.1),
                            ..continuous_audio()
                        }
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
                            NextButton,
                            button_messages,
                            TextButton::new(
                                vec![
                                    InputAction::PlaySound(String::from("click")),
                                    InputAction::ChangeState(next_state_vector),
                                ],
                                vec![KeyCode::Enter],
                                first_message
                            ),
                            TransientAudioPallet::new(
                                vec![(
                                    "click".to_string(),
                                    vec![
                                        TransientAudio::new(
                                            asset_server.load(
                                                "sounds/mech_click.ogg"
                                            ), 
                                            0.1, 
                                            true,
                                            1.0
                                        )
                                    ]
                                )]
                            ),
                            NextButton::transform(&windows)
                        ));
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
    mut text_query: Query<(&mut Text2d, &TextFrames)>,
) {
    let Some(button_entity) = next_state_button.entity() else {
        return;
    };

    let (mut text, frames) = match text_query.get_mut(button_entity) {
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

                text.0 = message.clone();
            }
        }
    }
}