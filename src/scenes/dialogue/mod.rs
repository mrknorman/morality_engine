
use std::collections::HashMap;

use bevy::{
    asset::AssetServer,
    audio::Volume,
    prelude::*
};

use crate::{
    systems::{
        audio:: {
            continuous_audio, 
            ContinuousAudio,
            ContinuousAudioPallet
        }, 
        interaction::InteractionPlugin, 
    },
    entities::graph::Graph,
    data::{
        character::{
            Character, 
            CharacterKey
        }, 
        states::{
            GameState, 
            Memory
        }, 
    },
    style::ui::IOPlugin
};

pub mod dialogue;
pub mod content;

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


#[derive(Component)]
#[require(Transform, Visibility)]
struct DialogueScene;

impl DialogueScene {

    pub fn setup(
        mut commands: Commands, 
        memory : Res<Memory>,
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
        
        commands.spawn(
            (
                DialogueScene,
                StateScoped(GameState::Dialogue),
                ContinuousAudioPallet::new(
                    vec![
                        ContinuousAudio{
                            key : DialogueSounds::Hum,
                            source : AudioPlayer::<AudioSource>(asset_server.load(
                                "./audio/effects/hum.ogg"
                            )),
                            settings : PlaybackSettings{
                                volume : Volume::new(0.1),
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
                                volume : Volume::new(0.5),
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
                                volume : Volume::new(0.3),
                                ..continuous_audio()
                            },
                            dilatable : false
                        }
                    ]
                )
            )
        ).with_children(
            |parent| {
                parent.spawn((
                    Dialogue::init(
                        &asset_server,
                        &memory.next_dialogue,
                        &character_map
                    ),
                    Transform::from_xyz(-500.0, 0.0, 1.0),
                    )
                );
                parent.spawn((
                    Graph::new(
                        45.0,
                        vec![2, 3, 2],
                        10.0,
                        20.0,
                        4.0,
                        2.0
                    ),
                    Transform::from_xyz(300.0, 0.0, 0.0)
                ));
            }
        );
    }
}

