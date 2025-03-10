use bevy::{
    audio::Volume, 
    prelude::*
};
use enum_map::{
    enum_map, 
    Enum
};

use crate::{
    ascii_fonts::{AsciiString, TextEmotion}, audio::{
        continuous_audio, BackgroundAudio, ContinuousAudio, ContinuousAudioPallet, OneShotAudio, OneShotAudioPallet, TransientAudio, TransientAudioPallet
    }, background::Background, colors::{DIM_BACKGROUND_COLOR, MENU_COLOR}, common_ui::NextButton, game_states::GameState, interaction::{ActionPallet, Draggable, InputAction}, sprites::window::WindowTitle, stats::GameStats, text::{TextButton, TextRaw, WindowedTable}, track::Track, train::{Train, TrainState, STEAM_TRAIN}
};


struct Ending {
    name : String,
    description : String
}

pub struct EndingScenePlugin;
impl Plugin for EndingScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Ending), 
            EndingScene::setup
        );
        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
    
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndingSounds {
    Wind,
	Static,
	Office,
    Click
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndingActions {
    ResetGame
}

impl std::fmt::Display for EndingActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Component)]
#[require(Transform, Visibility)]
struct EndingScene;

impl EndingScene{

    const TITLE_TRANSLATION : Vec3 = Vec3::new(-380.0, 225.0, 0.5);
    const TRAIN_TRANSLATION: Vec3 = Vec3::new(110.0, -35.0, 0.5);
    const TRACK_DISPLACEMENT: Vec3 = Vec3::new(-120.0, -30.0, 0.5);
    const RESULTS_TRANSLATION : Vec3 = Vec3::new(220.0, 130.0, 1.0);

    fn setup(
        mut commands: Commands, 
        stats : Res<GameStats>,
        asset_server: Res<AssetServer>
    ) {
        commands.spawn(
            (
                EndingScene,
                StateScoped(GameState::Ending),
            )
        ).with_children(
            |parent| {
                parent.spawn((
                    BackgroundAudio,
                    ContinuousAudioPallet::new(
                        vec![
                            ContinuousAudio{
                                key : EndingSounds::Wind,
                                source : AudioPlayer::<AudioSource>(
                                    asset_server.load(
                                        "./audio/effects/desert_wind.ogg"
                                    )
                                ), 
                                settings : PlaybackSettings{
                                    volume : Volume::new(1.0),
                                    ..continuous_audio()
                                },
                                dilatable : true
                            },
                            ContinuousAudio{
                                key : EndingSounds::Static,
                                source : AudioPlayer::<AudioSource>(
                                    asset_server.load(
                                        "./audio/effects/static.ogg"
                                    )
                                ), 
                                settings : PlaybackSettings{
                                    volume : Volume::new(0.1),
                                    ..continuous_audio()
                                },
                                dilatable : true
                            },
                            ContinuousAudio{
                                key : EndingSounds::Office,
                                source : AudioPlayer::<AudioSource>(
                                    asset_server.load(
                                        "./audio/effects/office.ogg"
                                    )
                                ), 
                                settings : PlaybackSettings{
                                    volume : Volume::new(0.2),
                                    ..continuous_audio()
                                },
                                dilatable : true
                            }
                        ]
                    )
                ));

                parent.spawn(
                    OneShotAudioPallet::new(
                        vec![
                            OneShotAudio{
                                source: asset_server.load("./audio/effects/game_over.ogg"),
                                volume : 0.4,
                                persistent : true,
                                dilatable: false
                            }
                        ]
                    )
                );

                parent.spawn((
                    Draggable::default(),
                    WindowedTable{
                        title : Some(WindowTitle{
                            text : String::from("Overall Results"),
                            ..default()
                        }),
                        ..default()
                    },
                    stats.to_table(),
                    Transform::from_translation(Self::RESULTS_TRANSLATION)
                ));

                parent.spawn((         
                    Background::load_from_json(
                        "text/backgrounds/desert.json",	
                        0.00002,
                        0.0
                    ),
                    TextColor(DIM_BACKGROUND_COLOR)
                ));

                parent.spawn((
                    Track::new(600),
                    TextColor(DIM_BACKGROUND_COLOR),
                    Transform::from_translation(Self::TRAIN_TRANSLATION + Self::TRACK_DISPLACEMENT)
                ));

                parent.spawn(
                    (
                        AsciiString(
                            "FALSE START".to_string()
                        ),
                        TextEmotion::Afraid,
                        TextColor(MENU_COLOR),
                        Transform::from_translation(Self::TITLE_TRANSLATION)
                    )
                );

                parent.spawn(
                    (                    
                        Train::new(STEAM_TRAIN),
                        TrainState::Wrecked,
                        Transform::from_translation(Self::TRAIN_TRANSLATION),
                        TextColor(MENU_COLOR),
                    )  
                );

                parent.spawn(
                    (
                        NextButton,
                        TextColor(MENU_COLOR),
                        TextButton::new(
                            vec![
                                EndingActions::ResetGame
                            ],
                            vec![KeyCode::Enter],
                            "[Click Here or Press Enter to Fade into Oblivion]",
                        ),
                        ActionPallet::<EndingActions, EndingSounds>(
                            enum_map!(
                                EndingActions::ResetGame => vec![
                                    InputAction::PlaySound(
                                        EndingSounds::Click
                                    ),
                                    InputAction::ResetGame
                                ]
                            )
                        ),
                        TransientAudioPallet::new(
                            vec![(
                                EndingSounds::Click,
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
                );
            }
        );
    }
}