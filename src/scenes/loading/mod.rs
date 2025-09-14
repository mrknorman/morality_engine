use std::{time::Duration, vec};
use bevy::{
    prelude::*,
    audio::Volume
};
use content::{LoadingBarMessages, LoadingButtonMessages};
use enum_map::{
    enum_map, 
    Enum
};

use crate::{
    data::states::GameState, entities::text::{
        TextButton,
        TextFrames 
    }, style::{
        common_ui::{
            NextButton,
            NextButtonConfig
        }, 
        ui::IOPlugin, 
    }, systems::{
        audio::{
            continuous_audio, 
            one_shot_audio, 
            ContinuousAudio, 
            ContinuousAudioPallet, 
            MusicAudio, 
            NarrationAudio, 
            OneShotAudio, 
            OneShotAudioPallet, 
            TransientAudio, 
            TransientAudioPallet
        }, cascade::{Cascade, CascadePlugin}, colors::{AlphaTranslation, DIM_BACKGROUND_COLOR}, inheritance::BequeathTextAlpha, interaction::{
            ActionPallet, 
            InputAction, 
            InteractionPlugin
        }, scheduling::{
            TimerConfig, 
            TimerPallet,
            TimerStartCondition, 
            TimingPlugin
        }
    }
};

mod loading_bar;
use loading_bar::{LoadingBar, LoadingBarPlugin};

use super::SceneQueue;

pub mod content;


pub struct LoadingScenePlugin;
impl Plugin for LoadingScenePlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            OnEnter(
                GameState::Loading
            ), 
            LoadingScene::setup
        )
        .add_systems(
            Update,
            (
                LoadingScene::spawn_delayed_children,
                LoadingScene::update_button_text,
            )
                .run_if(in_state(GameState::Loading)),
        );

        if !app.is_plugin_added::<CascadePlugin>() {
            app.add_plugins(CascadePlugin);
        }
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

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingActions {
    ExitLoading
}

impl std::fmt::Display for LoadingActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingSounds {
	Hum,
	Office,
    Click
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingEvents {
    Music,
	Narration,
	Button,
    UpdateButton
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct LoadingScene;

impl LoadingScene {

    fn setup(
        mut commands: Commands,
        queue : Res<SceneQueue>,
        asset_server: Res<AssetServer>
    ) {

        commands.spawn((
            queue.current,
            LoadingScene,
            StateScoped(GameState::Loading),
            TimerPallet::new(
                vec![
                    (
                        LoadingEvents::Music,
                        TimerConfig::new(
                            TimerStartCondition::Immediate, 
                            2.0,
                            None
                        )
                    ),
                    (
                        LoadingEvents::Narration,
                        TimerConfig::new(
                            TimerStartCondition::Immediate, 
                            3.0,
                            None
                        )
                    ),
                    (
                        LoadingEvents::Button,
                        TimerConfig::new(
                            TimerStartCondition::Immediate, 
                            5.0,
                            None
                        )
                    ),
                    (
                        LoadingEvents::UpdateButton,
                        TimerConfig::new(
                            TimerStartCondition::OnNarrationEnd, 
                            0.5,
                            Some(20.0)
                        )
                    )
                ]
            ),
            ContinuousAudioPallet::new(
                vec![
                    ContinuousAudio{
                        key : LoadingSounds::Hum,
                        source : AudioPlayer::<AudioSource>(asset_server.load(
                            "./audio/effects/hum.ogg"
                        )),
                        settings : PlaybackSettings{
                            volume : Volume::Linear(0.1),
                            ..continuous_audio()
                        },
                        dilatable : false
                    },
                    ContinuousAudio{
                        key : LoadingSounds::Office,
                        source : AudioPlayer::<AudioSource>(asset_server.load(
                            "./audio/effects/office.ogg"
                        )),
                        settings : PlaybackSettings{
                            volume : Volume::Linear(0.5),
                            ..continuous_audio()
                        },
                        dilatable : true
                    }
                ]
            ),
            OneShotAudioPallet::new(
                vec![
                    OneShotAudio{
                        source: asset_server.load(
                                "./audio/effects/startup_beep.ogg"
                            ),
                        persistent : true,
                        ..default()
                    },
                ]
            ),
            children![
                LoadingBar::load(
                    LoadingBarMessages::Lab0intro
                )
            ]
        ));
    }

    fn spawn_delayed_children(
            mut commands: Commands,
            loading_query: Query<(Entity, &TimerPallet<LoadingEvents>), With<LoadingScene>>,
            asset_server: Res<AssetServer>
        ) {

        for (entity, timers) in loading_query.iter() {
            // Handle narration timer

            if timers.0[LoadingEvents::Narration].just_finished() {
                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn((
                        NarrationAudio,
                        AudioPlayer::<AudioSource>(asset_server.load(
                            "./audio/effects/intro_lilly_elvenlabs.ogg", 
                        )),
                        PlaybackSettings{
                            volume : Volume::Linear(1.0),
                            ..one_shot_audio()
                        }
                    ));
                });
            }

            if timers.0[LoadingEvents::Music].just_finished() {
                commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn((
                        MusicAudio,
                        AudioPlayer::<AudioSource>(asset_server.load(
                            "./audio/music/a_friend.ogg"
                        )),
                        PlaybackSettings{
                            paused : false,
                            volume : Volume::Linear(0.1),
                            ..continuous_audio()
                        }
                    ));

                    parent.spawn((
                        Cascade::default(),
                        BequeathTextAlpha,
                        AlphaTranslation::new(
                            DIM_BACKGROUND_COLOR.alpha(),
                            Duration::from_secs_f32(1.0),
                            false
                        ),
                        TextColor(DIM_BACKGROUND_COLOR)
                    ));
                });
            }

            // Handle button timer
            if timers.0[LoadingEvents::Button].just_finished() {
                let button_messages = TextFrames::load(
                    LoadingButtonMessages::Lab0intro.content()
                );

                if let Ok(button_messages) = button_messages {
                    commands.entity(entity).with_children(|parent| {

                        let first_message = button_messages.frames[0].clone();
                        parent.spawn((
                            NextButton,
                            button_messages,
                            TextButton::new(
                                vec![
                                    LoadingActions::ExitLoading,
                                ],
                                vec![KeyCode::Enter],
                                first_message
                            ),
                            ActionPallet::<LoadingActions, LoadingSounds>(
                                enum_map!(
                                    LoadingActions::ExitLoading => vec![
                                        InputAction::NextScene,
                                        InputAction::PlaySound(LoadingSounds::Click),
                                    ]
                                )
                            ),
                            TransientAudioPallet::new(
                                vec![(
                                    LoadingSounds::Click,
                                    vec![
                                        TransientAudio::new(
                                            asset_server.load(
                                                "./audio/effects/mech_click.ogg"
                                            ), 
                                            0.1, 
                                            true,
                                            1.0,
                                            true
                                        )
                                    ]
                                )]
                            )
                        ));
                    });
                }
            }
        }
    }

    fn update_button_text(
        next_state_button: Res<NextButtonConfig>,
        loading_query: Query<(&TimerPallet<LoadingEvents>, &LoadingScene)>,
        mut text_query: Query<(&mut Text2d, &TextFrames)>,
    ) {
        let Some(button_entity) = next_state_button.0 else {
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

        if timers.0[LoadingEvents::UpdateButton].just_finished() {
            let index = timers.0[LoadingEvents::UpdateButton].times_finished() as usize;
            if let Some(message) = frames.frames.get(index) {

                text.0 = message.clone();
            }
        }
    }
}