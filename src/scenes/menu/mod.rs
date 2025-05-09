use bevy::{audio::Volume, prelude::*};
use enum_map::{
    Enum, 
    enum_map
};

use crate::{
    data::states::MainState,
    systems::{
        audio::{
            continuous_audio, 
            BackgroundAudio, 
            ContinuousAudio, 
            ContinuousAudioPallet, 
            MusicAudio, 
            TransientAudio, 
            TransientAudioPallet
        }, 
        backgrounds::{
            Background, 
            BackgroundPlugin,
            content::BackgroundTypes
        }, 
        colors::{
            DIM_BACKGROUND_COLOR, 
            MENU_COLOR
        },
        interaction::{ 
            ActionPallet, 
            InputAction, 
            InteractionPlugin
        }
    }, 
    entities::{
        large_fonts::{
            AsciiPlugin, 
            AsciiString, TextEmotion
        }, 
        text::{
            TextButton, 
            TextRaw
        }, 
        train::{
            Train, 
            TrainPlugin, 
            content::TrainTypes
        },
        track::Track, 
    },
    style::{
        common_ui::NextButton, 
        ui::IOPlugin
    }
};

use super::SceneQueue;

pub struct MenuScenePlugin;
impl Plugin for MenuScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainState::Menu), 
            MenuScene::setup
        );
        if !app.is_plugin_added::<TrainPlugin>() {
            app.add_plugins(TrainPlugin);
        }
        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
        if !app.is_plugin_added::<AsciiPlugin>() {
            app.add_plugins(AsciiPlugin);
        }
        if !app.is_plugin_added::<BackgroundPlugin>() {
			app.add_plugins(BackgroundPlugin);
		}
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuSounds {
	Static,
	Office,
    Click
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuActions {
    EnterGame
}

impl std::fmt::Display for MenuActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct MenuScene;

impl MenuScene {

    const TITLE_TRANSLATION : Vec3 = Vec3::new(-380.0, 225.0, 1.0);
    const TRAIN_TRANSLATION: Vec3 = Vec3::new(110.0, -35.0, 0.5);
    const TRACK_DISPLACEMENT: Vec3 = Vec3::new(-120.0, -30.0, 0.4);
    const SIGNATURE_TRANSLATION : Vec3 = Vec3::new(0.0, -100.0, 1.0);

    fn setup(
        mut commands: Commands, 
        queue : Res<SceneQueue>,
        asset_server: Res<AssetServer>
    ) {
        commands.spawn(
            (
                queue.current,
                MenuScene,
                StateScoped(MainState::Menu),
                children![
                    (
                        BackgroundAudio,
                        ContinuousAudioPallet::new(
                            vec![
                                ContinuousAudio{
                                    key : MenuSounds::Static,
                                    source : AudioPlayer::<AudioSource>(
                                        asset_server.load(
                                            "./audio/effects/static.ogg"
                                        )
                                    ), 
                                    settings : PlaybackSettings{
                                        volume : Volume::Linear(0.1),
                                        ..continuous_audio()
                                    },
                                    dilatable : true
                                },
                                ContinuousAudio{
                                    key : MenuSounds::Office,
                                    source : AudioPlayer::<AudioSource>(
                                        asset_server.load(
                                            "./audio/effects/office.ogg"
                                        )
                                    ), 
                                    settings : PlaybackSettings{
                                        volume : Volume::Linear(0.5),
                                        ..continuous_audio()
                                    },
                                    dilatable : true
                                }
                            ]
                        )
                    ),
                    (         
                        Background::new(
                            BackgroundTypes::Desert,	
                            0.00002,
                            -0.5
                        ),
                        TextColor(DIM_BACKGROUND_COLOR)
                    ),
                    (
                        Track::new(600),
                        TextColor(DIM_BACKGROUND_COLOR),
                        Transform::from_translation(Self::TRAIN_TRANSLATION + Self::TRACK_DISPLACEMENT)
                    ),
                    (
                        MusicAudio,
                        AudioPlayer::<AudioSource>(asset_server.load(
                            "./audio/music/the_last_decision.ogg", 
                        )),
                        PlaybackSettings{
                            volume : Volume::Linear(0.3),
                            ..continuous_audio()
                        }
                    ),
                    (
                        TextEmotion::Happy,
                        AsciiString(
                            "THE TROLLEY\n  ALGORITHM".to_string()
                        ),
                        TextColor(MENU_COLOR),
                        Transform::from_translation(Self::TITLE_TRANSLATION)
                    ),
                    (                    
                        Train(TrainTypes::SteamTrain),
                        Transform::from_translation(Self::TRAIN_TRANSLATION),
                        TextColor(MENU_COLOR),
                    ), 
                    (
                        TextRaw,
                        Text2d::new(
                            "A game by Michael Norman"
                        ),
                        Transform::from_translation(Self::SIGNATURE_TRANSLATION)
                    ),
                    (
                        NextButton,
                        TextColor(MENU_COLOR),
                        TextButton::new(
                            vec![
                                MenuActions::EnterGame
                            ],
                            vec![KeyCode::Enter],
                            "[Click Here or Press Enter to Begin]",
                        ),
                        ActionPallet::<MenuActions, MenuSounds>(
                            enum_map!(
                                MenuActions::EnterGame => vec![
                                    InputAction::PlaySound(
                                        MenuSounds::Click
                                    ),
                                    InputAction::NextScene
                                ]
                            )
                        ),
                        TransientAudioPallet::new(
                            vec![(
                                MenuSounds::Click,
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
                        ),
                    )
                ]
            ),
        );
    }
}