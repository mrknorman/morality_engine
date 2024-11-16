
use std::collections::HashMap;

use bevy::{
    asset::AssetServer,
    prelude::*
};

use crate::{
    audio::{
        ContinuousAudioPallet, 
        ContinuousAudio
    },
    game_states::GameState,
    character::Character,
    io::IOPlugin, 
    interaction::InteractionPlugin,
    graph::GraphBundle
};



mod dialogue;
use dialogue::{
    DialoguePlugin,
    DialogueBundle
};

pub struct DialogueScreenPlugin;
impl Plugin for DialogueScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(
                GameState::Dialogue
            ), 
            setup_dialogue
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
pub struct DialogueRoot;

pub fn setup_dialogue(
        mut commands: Commands, 
        asset_server : Res<AssetServer>
    ) {

    let character_map = HashMap::from([
        (
            String::from("creator"),  
            Character::new(
                "creator", 
                "sounds/typing.ogg",
                Color::srgba(0.4039, 0.9490, 0.8196, 1.0)
            )
        )
    ]);
    
    commands.spawn(
        (
            DialogueRoot,
            StateScoped(GameState::Dialogue),
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
                            0.1,
                            false
                        ),
                    ),
                    (
                        "music".to_string(), 
                        ContinuousAudio::new(
                            &asset_server,
                            "./music/trolley_wires.ogg",
                            0.3,
                            false
                        )
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
            )
        )
    ).with_children(
        |parent| {
            parent.spawn(
                DialogueBundle::load(
                    "./text/lab_1.json",
                    &asset_server,
                    &character_map
                )
            );
            parent.spawn(
                GraphBundle::new(
                    3,
                    45.0,
                    vec![2, 3, 2],
                    10.0,
                    20.0,
                    4.0,
                    Vec3::new(350.0, 0.0, 0.0),
                    1.5
                )
            );
        }
    );
}