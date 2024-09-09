
use std::{
    path::PathBuf, 
    collections::
    HashMap
};

use bevy::{
    asset::AssetServer, 
    text::{
        BreakLineOn, 
        Text2dBounds
    }, 
    sprite::Anchor, 
    prelude::*
};

use crate::{
    audio::{
        ContinuousAudioPallet, 
        ContinuousAudio
    },
    game_states::GameState,
    character::Character,
    io::{
        BottomAnchor, 
        IOPlugin
    }, 
    text::TextButtonBundle, 
    interaction::{
        InteractionPlugin,
        InputAction
    },
    common_ui::NextButtonBundle
};

mod dialogue;
use dialogue::{
    DialogueLine,
    DialoguePlugin,
    Dialogue,
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
        ).add_systems(
            Update,
            (
                DialogueLine::play
            )
            .run_if(
                in_state(
                    GameState::Dialogue
                )
            ),
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
        asset_server : Res<AssetServer>,
        windows: Query<&Window>
    ) {

    let character_map = HashMap::from([
        (
            String::from("creator"),  
            Character::new(
                "creator", 
                "sounds/typing.ogg"
            )
        )
    ]);

    let dialogue = Dialogue::load(
        PathBuf::from("./text/lab_1.json"),
        &asset_server,
        character_map.clone()
    );

    let text_section = TextSection::new(
        "",
        TextStyle {
            // This font is loaded and will be used instead of the default font.
            font_size: 15.0,
            ..default()
        });

    let mut line_vector : Vec<TextSection> = vec![];
    for line in dialogue.lines {
        commands.spawn(line);
        line_vector.push(text_section.clone());
    }

    let box_size = Vec2::new(500.0, 2000.0);

    let button_distance = 100.0;
    let window = windows.get_single().unwrap();
    let screen_height = window.height();
    let button_y = -screen_height / 2.0 + button_distance; 
    let button_translation: Vec3 = Vec3::new(0.0, button_y, 1.0);

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
                            0.1
                        ),
                    ),
                    (
                        "music".to_string(), 
                        ContinuousAudio::new(
                            &asset_server,
                            "./music/trolley_wires.ogg",
                            0.3,
                        )
                    ),
                    (
                        "office".to_string(),
                        ContinuousAudio::new(
                            &asset_server, 
                            "./sounds/office.ogg", 
                            0.5
                        ),
                    )
                ]
            )
        )
    ).with_children(
        |parent| {

            parent.spawn(
                (
                Text2dBundle {
                    text : Text {
                        sections : line_vector,
                        justify : JustifyText::Left, 
                        linebreak_behavior: BreakLineOn::WordBoundary
                    },
                    text_2d_bounds : Text2dBounds {
                        size: box_size,
                    },
                    transform: Transform::from_xyz(-500.0,0.0, 1.0),
                    text_anchor : Anchor::CenterLeft,
                    ..default()
                },
                DialogueBundle::load(
                    "./text/lab_1.json",
                    &asset_server,
                    character_map
                ))
            );
        }
    );
}

/*
fn update_button_text(
    next_state_button: Res<NextStateButton>,
    loading_query: Query<(&TimerPallet, &LoadingRoot)>,
    mut text_query: Query<(&mut Text, &TextFrames, &mut Clickable)>,
) {
    let Some(button_entity) = next_state_button.entity else {
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
*/