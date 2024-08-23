
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
    timing::{
        TimerPallet, 
        TimingPlugin, 
        TimerStartCondition
    }
};

mod dialogue;
use dialogue::{
    DialogueLine,
    DialogueText, 
    Dialogue
};

pub struct DialoguePlugin;
impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(
                GameState::Dialogue
            ), 
            setup_dialogue
        ).add_systems(
            Update,
            (
                DialogueLine::play, 
                DialogueLine::typewriter_effect
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
        if !app.is_plugin_added::<TimingPlugin>() {
            app.add_plugins(TimingPlugin);
        }
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
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
        character_map
    );

    let num_lines: usize = dialogue.lines.len();

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
    let button_translation: Vec3 = Vec3::new(500.0, button_y, 1.0);

    commands.spawn(
        (
            DialogueRoot,
            StateScoped(GameState::Dialogue),
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
            DialogueText {
                current_line_index : 0,
                total_num_lines : num_lines
            },
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
            parent.spawn((
                BottomAnchor::new(button_distance),
                TextButtonBundle::new(
                    &asset_server,
                    vec![
                        InputAction::PlaySound(String::from("click"))
                    ],
                    vec![KeyCode::Enter],
                    "[Click Here or Press Enter to Ignore Your Inner Dialogue]",
                    button_translation,
                ),
            )); 
        }
    );
}