use bevy::{
    audio::Volume, prelude::*, sprite::Anchor, text::TextBounds
};
use enum_map::{
    enum_map, 
    Enum
};
use serde::{
    Deserialize,
    Serialize
};

use crate::{
    data::{
        states::GameState, stats::GameStats 
    }, entities::{
        large_fonts::{
            AsciiString, 
            TextEmotion
        }, 
        sprites::window::WindowTitle, 
        text::{
            TextButton, TextWindow, WindowedTable
        },
        track::Track, 
        train::{
            content::TrainTypes, Train, TrainState
        }
    }, style::common_ui::NextButton, systems::{
        audio::{
            continuous_audio, one_shot_audio, BackgroundAudio, ContinuousAudio, ContinuousAudioPallet, NarrationAudio, OneShotAudio, OneShotAudioPallet, TransientAudio, TransientAudioPallet
        }, backgrounds::{
            content::BackgroundTypes, Background
        }, colors::{
            DIM_BACKGROUND_COLOR, 
            MENU_COLOR, PRIMARY_COLOR
        }, interaction::{
            ActionPallet, 
            Draggable, 
            InputAction
        }, scheduling::{TimerConfig, TimerPallet, TimerStartCondition}
    }
};

pub mod content;

use content::EndingScene;

use super::{Scene, SceneQueue};

#[derive(Component, Clone, Debug, Serialize, Deserialize, Resource)]
pub struct Ending{
    pub name: String,
    pub description: String,
    pub narration: String,
    pub narration_path: String,
}

impl Ending{
    pub fn new(ending_content: EndingScene) -> Self {
        let json_content = ending_content.content();
        serde_json::from_str(json_content).expect("Failed to pass embedded JSON!")
    }
}

impl Default for Ending{
    fn default() -> Self {
        Self{
            name : String::new(),
            description : String::new(),
            narration : String::new(),
            narration_path : String::new(),
        }    
    }
}

pub struct EndingScenePlugin;
impl Plugin for EndingScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Ending), 
            EndingScene::setup
        ).add_systems(
            Update, 
            EndingScene::spawn_delayed_children
        )
        .insert_resource(Ending::default());
        
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

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndingEvents {
    Narration,
	Button
}

impl std::fmt::Display for EndingActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl EndingScene{

    const TITLE_TRANSLATION : Vec3 = Vec3::new(-380.0, 225.0, 0.5);
    const TRAIN_TRANSLATION: Vec3 = Vec3::new(110.0, -35.0, 0.5);
    const TRACK_DISPLACEMENT: Vec3 = Vec3::new(-120.0, -30.0, 0.5);
    const RESULTS_TRANSLATION : Vec3 = Vec3::new(220.0, 130.0, 1.0);

    fn setup(
        mut commands: Commands, 
        stats : Res<GameStats>,
        queue : Res<SceneQueue>,
        asset_server: Res<AssetServer>
    ) {
		let scene = queue.current;
		let ending = match scene {
			Scene::Ending(content) => {
				Ending::new(content)
			},
			_ => panic!("Scene is not dilemma!") 
		};

        commands.insert_resource(ending.clone());

        commands.spawn(
            (
                queue.current,
                DespawnOnExit(GameState::Ending),
                TimerPallet::new(
					vec![
						(
							EndingEvents::Narration,
							TimerConfig::new(
								TimerStartCondition::Immediate, 
								1.0,
								None
							)
						),
						(
							EndingEvents::Button,
							TimerConfig::new(
								TimerStartCondition::Immediate, 
								2.0,
								None
							)
						)
					]
				)
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
                                    volume : Volume::Linear(1.0),
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
                                    volume : Volume::Linear(0.06),
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
                                    volume : Volume::Linear(0.2),
                                    ..continuous_audio()
                                },
                                dilatable : true
                            }
                        ]
                    )
                ));

                parent.spawn((
					TextWindow{
						title : Some(WindowTitle{
							text : format!("Description: {}" , ending.name.clone()),
							..default()
						}),
						..default()
					},
					TextBounds {
						width : Some(400.0), 
						height : None
					},
					Draggable::default(),
					TextColor(PRIMARY_COLOR),
					Text2d::new(&ending.description),
					TextFont{
						font_size : 12.0,
						..default()
					},
					Anchor::TOP_LEFT,
					Transform::from_xyz(-600.0,200.0, 2.0)
				));	

                parent.spawn(
                    OneShotAudioPallet::new(
                        vec![
                            OneShotAudio{
                                source: asset_server.load("./audio/effects/game_over.ogg"),
                                volume : 0.4,
                                persistent : true,
                                ..default()
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
                    Background::new(
                        BackgroundTypes::Desert,	
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
                        Train(TrainTypes::SteamTrain),
                        TrainState::Wrecked,
                        Transform::from_translation(Self::TRAIN_TRANSLATION),
                        TextColor(MENU_COLOR),
                    )  
                );
            }
        );
    }

    fn spawn_delayed_children(
		mut commands: Commands,
		ending : Res<Ending>,
		loading_query: Query<(Entity, &TimerPallet<EndingEvents>)>,
		asset_server: Res<AssetServer>
	) {

		for (entity, timers) in loading_query.iter() {

			if timers.0[EndingEvents::Narration].just_finished() {
				commands.entity(entity).with_children(
					|parent| {
						parent.spawn((
							NarrationAudio,
							AudioPlayer::<AudioSource>(asset_server.load(
								ending.narration_path.clone(),
							)),
							PlaybackSettings{
								paused : false,
								volume : Volume::Linear(1.0),
								..one_shot_audio()
							}
						));
				});
			}

			// Handle narration timer
			if timers.0[EndingEvents::Button].just_finished() {          
				commands.entity(entity).with_children(|parent| {
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
				});
			}
		}
	}
}