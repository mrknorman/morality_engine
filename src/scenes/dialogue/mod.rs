
use std::{collections::HashMap, time::Duration};

use bevy::{
    asset::AssetServer,
    audio::Volume,
    prelude::*
};

use crate::{
    data::{
        character::{
            Character, 
            CharacterKey
        }, 
        states::GameState, 
    }, entities::graph::Graph, scenes::{
        Scene, 
        SceneQueue
    }, style::ui::IOPlugin, systems::{
        audio:: {
            ContinuousAudio, ContinuousAudioPallet, continuous_audio
        }, cascade::Cascade, colors::{AlphaTranslation, DIM_BACKGROUND_COLOR}, inheritance::BequeathTextAlpha, interaction::InteractionPlugin 
    }
};

pub mod dialogue;
pub mod content;

use content::DialogueScene;
use dialogue::{
    Dialogue, 
    DialoguePlugin, 
    DialogueSounds
};

pub struct DialogueScenePlugin;
impl Plugin for DialogueScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(
                GameState::Dialogue
            ), 
            DialogueScene::setup
        );

        if !app.is_plugin_added::<InteractionPlugin>() {
            app.add_plugins(InteractionPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
        if !app.is_plugin_added::<DialoguePlugin>() {
            app.add_plugins(DialoguePlugin);
        }
    }
}

impl DialogueScene {
    pub fn setup(
        mut commands: Commands, 
        mut queue : ResMut<SceneQueue>,
        asset_server : Res<AssetServer>
    ) {

        let character_map = HashMap::from([
            (
                String::from("creator"),  
                Character::new(
                    "creator", 
                    "./audio/effects/typing.ogg",
                    Color::srgba(0.4039 * 3.0, 0.9490 * 3.0, 0.8196 * 3.0, 1.0),
                    CharacterKey::Creator
                )
            ),
            (
                String::from("child"),  
                Character::new(
                    "child", 
                    "./audio/effects/typing.ogg",
                    Color::srgba(0.635 * 3.0, 0.0* 3.0, 1.0 * 3.0, 1.0),
                    CharacterKey::Creator
                )
            )
        ]);

        let scene = queue.current;
        let dialogue: DialogueScene = match scene {
			Scene::Dialogue(content) => {
				content
			},
			_ => panic!("Scene is not dialogue!") 
		};

        let mut dialogue_vector= vec![dialogue];

        let next_scene = match queue.next {
            Some(Scene::Dialogue(_)) => {
                Some(queue.pop())
            },
            Some(_) | None => None,
        };
        match next_scene {
            Some(Scene::Dialogue(content)) => {
				dialogue_vector.push(content);
			},
			_ => ()
        };

        commands.spawn(
            (
                scene,
                DespawnOnExit(GameState::Dialogue),
                ContinuousAudioPallet::new(
                    vec![
                        ContinuousAudio{
                            key : DialogueSounds::Hum,
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
                            key : DialogueSounds::Office,
                            source : AudioPlayer::<AudioSource>(asset_server.load(
                                "./audio/effects/office.ogg"
                            )),
                            settings : PlaybackSettings{
                                volume : Volume::Linear(0.5),
                                ..continuous_audio()
                            },
                            dilatable : true
                        },
                        ContinuousAudio{
                            key : DialogueSounds::Music,
                            source : AudioPlayer::<AudioSource>(asset_server.load(
                                "./audio/music/trolley_wires.ogg"
                            )),
                            settings : PlaybackSettings{
                                volume : Volume::Linear(0.3),
                                ..continuous_audio()
                            },
                            dilatable : false
                        }
                    ]
                ),
                children![
                    (
                        Dialogue::init(
                            &asset_server,
                            &dialogue_vector,
                            &character_map
                        ),
                        Transform::from_xyz(-500.0, 0.0, 1.0),
                    ),
                    (
                        Graph::new(
                            45.0,
                            vec![2, 3, 2],
                            10.0,
                            20.0,
                            4.0,
                            2.0
                        ),
                        Transform::from_xyz(300.0, 0.0, 0.5)
                    ),
                    (
                        Cascade{
                            speed : 50.0,
                            visibility_speed : 0.1,
                            ..default()
                        },
                        BequeathTextAlpha,
                        AlphaTranslation::new(
                            DIM_BACKGROUND_COLOR.alpha(),
                            Duration::from_secs_f32(1.0),
                            false
                        ),
                        TextColor(DIM_BACKGROUND_COLOR)
                    )
                ]
            )
        );
    }
}

