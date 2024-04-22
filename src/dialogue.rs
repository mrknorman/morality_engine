
use std::{path::PathBuf, time::Duration};
use bevy::{asset::AssetServer, prelude::*};

#[derive(Component)]
pub struct Character {
    name : String,
    audio_source_file_path : PathBuf
}

impl Character {
    fn new(name : &str, audio_source_file_path : &str) -> Character {
    
        Character {
            name: String::from(name),
            audio_source_file_path: PathBuf::from(audio_source_file_path)
        }
    }
}

#[derive(Component)]
pub struct DialogueLine {
    raw_text : String,
    current_index : usize,
    playing : bool,
    started : bool,
    timer: Timer,
    char_duration_milis : u64
} 

impl DialogueLine {
    fn new(raw_text : &str) -> DialogueLine {
        DialogueLine {
            raw_text : String::from(raw_text),
            current_index : 0,
            playing : false,
            started : false,
            timer : Timer::new(Duration::from_millis(0), TimerMode::Repeating),
            char_duration_milis : 30
        }
    }
}

pub fn play_dialogue (
        mut query: Query<(&Character, &mut DialogueLine, &mut Text, &mut AudioSink)>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {

    if keyboard_input.just_pressed(KeyCode::Enter) {

        for (character, mut dialogue, mut text, audio) in query.iter_mut() {
            if !dialogue.started {
                text.sections[0].value += character.name.as_str();
                text.sections[0].value += ":\n    ";
                audio.play();
                dialogue.playing = true;
                dialogue.timer = Timer::new(
                    Duration::from_millis(dialogue.char_duration_milis), 
                    TimerMode::Repeating
                );
                dialogue.started = true;
            }
        }
    }
}

pub fn typewriter_effect(
        mut query: Query<(&mut DialogueLine, &mut Text, &mut AudioSink)>,
        time: Res<Time>
    ) {
    
    // For each character in the dialogue, update all texts in the query
    for (mut entity, mut text, audio) in query.iter_mut() {

        if entity.playing {

            entity.timer.tick(time.delta());

            if entity.timer.finished() {

                let char = entity.raw_text.chars().nth(entity.current_index);

                match char {
                    Some(_) => text.sections[0].value.push(char.unwrap()), // Push character directly to each text,
                    None => {
                        audio.stop();
                        entity.playing = false;
                    }
                }

                entity.current_index += 1; 
            }
        }
    }
}
#[derive(Bundle)]
struct DialogueLineBundle {
    character : Character,
    audio : AudioBundle,
    line : DialogueLine,
    text : TextBundle,
}

impl DialogueLineBundle {

    fn new(
        character : Character,
        line : DialogueLine,
        asset_server : Res<AssetServer>
    ) -> DialogueLineBundle {

        let audio_source_file_path = character.audio_source_file_path.clone();

        DialogueLineBundle {
            character : character,
            audio : AudioBundle {
                    source: asset_server.load(audio_source_file_path),
                    settings : PlaybackSettings {
                        paused : true,
                        mode:  bevy::audio::PlaybackMode::Loop,
                        ..default()
                    }
            },
            line : line,
            // Create a TextBundle that has a Text with a single section.
            text : TextBundle::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font_size: 12.0,
                    ..default()
                }
            ) // Set the justification of the Text
            .with_text_justify(JustifyText::Left)
            // Set the style of the TextBundle itself.
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            })
        }
    }
}


#[derive(Component)]
struct Dialogue {
    lines : Vec<DialogueLineBundle>
}

#[derive(Bundle)]
struct DialogueBundle {
    text : TextBundle,
    dialogue : Dialogue
}




pub fn conversation(
        mut commands: Commands, 
        asset_server : Res<AssetServer>
    ) {
    
    commands.spawn(
        DialogueLineBundle::new(
            Character::new("The Creator", "sounds/typing.ogg"),
            DialogueLine::new("Wake Up!"),
            asset_server
        )
    );
}