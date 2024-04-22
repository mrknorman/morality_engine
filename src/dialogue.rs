
use std::{path::PathBuf, time::Duration, fs::File, io::BufReader, collections::HashMap};
use bevy::{asset::AssetServer, prelude::*};
use serde::{Serialize, Deserialize};

#[derive(Component, Clone)]
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
    index : usize,
    raw_text : String,
    instructon_text : String,
    current_index : usize,
    playing : bool,
    started : bool,
    timer: Timer,
    char_duration_milis : u64
} 

impl DialogueLine {
    fn new(raw_text : &str, instructon_text : &str, index : usize) -> DialogueLine {
        DialogueLine {
            index : index,
            raw_text : String::from(raw_text),
            instructon_text : String::from(instructon_text),
            current_index : 0,
            playing : false,
            started : false,
            timer : Timer::new(Duration::from_millis(0), TimerMode::Repeating),
            char_duration_milis : 30
        }
    }
}

pub fn play_dialogue (
        mut query_line: Query<(&Character, &mut DialogueLine, &mut AudioSink)>,
        mut query_text: Query<(&mut Text, &DialogueText)>,
        keyboard_input: Res<ButtonInput<KeyCode>>
    ) {

    let mut text_entity = query_text.iter_mut().next(); 

    if keyboard_input.just_pressed(KeyCode::Enter) {

        for (character, mut line, audio) in query_line.iter_mut() {
            if !line.started {
                match text_entity.as_ref() {
                    Some(_) => {
                        let (text, dialogue) = text_entity.as_mut().unwrap();

                        if line.index == dialogue.current_line_index {
                            text.as_mut().sections[line.index].value += character.name.as_str();
                            text.as_mut().sections[line.index].value += ":\n    ";
                        
                            audio.play();
                            line.playing = true;
                            line.timer = Timer::new(
                                Duration::from_millis(line.char_duration_milis), 
                                TimerMode::Repeating
                            );
                            line.started = true;
                        }
                    },
                    None => {}
                }
            } else {
                line.timer = Timer::new(
                    Duration::from_millis(line.char_duration_milis / 4), 
                    TimerMode::Repeating
                );
                audio.set_speed(4.0);
            }
        }
    }
}

pub fn typewriter_effect(
        mut query_line: Query<(&mut DialogueLine,  &mut AudioSink)>,
        mut query_text: Query<(&mut Text, &mut DialogueText)>,
        time: Res<Time>
    ) {

    let mut text_entity = query_text.iter_mut().next(); 
    
    // For each character in the dialogue, update all texts in the query
    for (mut entity, audio) in query_line.iter_mut() {

        if entity.playing {

            entity.timer.tick(time.delta());

            if entity.timer.finished() {

                let char = entity.raw_text.chars().nth(entity.current_index);
                
                match text_entity.as_ref() {
                    Some(_) => { 
                        let (text, dialogue) = text_entity.as_mut().unwrap();
                        
                        if entity.index == dialogue.current_line_index {
                            match char {
                                Some(_) => text.as_mut().sections[entity.index].value.push(char.unwrap()), // Push character directly to each text,
                                None => {
                                    audio.stop();
                                    entity.playing = false;
                                    dialogue.current_line_index += 1;
                                    text.as_mut().sections[entity.index].value.push('\n')
                                }
                            }

                            entity.current_index += 1; 
                        }
                    },
                    None => {}
                }
            }
        }
    }
}
#[derive(Bundle)]
struct DialogueLineBundle {
    character : Character,
    audio : AudioBundle,
    line : DialogueLine,
}

impl DialogueLineBundle {

    fn new(
        character : Character,
        line : DialogueLine,
        asset_server : &Res<AssetServer>
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
            line : line
        }
    }
}

pub struct Dialogue {
	lines : Vec<DialogueLineBundle>
}

#[derive(Serialize, Deserialize, Debug)]
struct DialogueLineLoader {
    character: String,
    dialogue: String,
    instruction: String
}

#[derive(Serialize, Deserialize, Debug)]
struct DialogueLoader {
    lines : Vec<DialogueLineLoader>
}

impl Dialogue {

	pub fn load(
            file_path: PathBuf,
            asset_server : Res<AssetServer>,
            character_map : HashMap<String, Character>
        ) -> Dialogue {
        
        // Open the file in read-only mode with a buffered reader
        let file = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);

        // Deserialize the JSON data into a Dialogue struct
        let loaded_dialogue: DialogueLoader = DialogueLoader {
			lines : serde_json::from_reader(reader).unwrap()
		};

        let mut line_vec = vec![];
        for (index, line) in loaded_dialogue.lines.iter().enumerate() {

            let character_key = line.character.as_str();
            let character  = character_map.get(character_key)
                .expect("Character not found in map") // This replaces unwrap() for better error handling
                .clone(); // This clones the Character object

            line_vec.push(
                DialogueLineBundle::new(
                    character,
                    DialogueLine::new(
                        line.dialogue.as_str(), 
                        line.instruction.as_str(), 
                        index
                    ),
                    &asset_server
                )
            );
        };

        Dialogue{
            lines : line_vec
        }
    }
}

#[derive(Component)]
pub struct DialogueText {
    current_line_index : usize
}

pub fn spawn_conversation(
        mut commands: Commands, 
        asset_server : Res<AssetServer>
    ) {


    let character_map = HashMap::from([
        (String::from("The Creator"),  Character::new("The Creator", "sounds/typing.ogg"))
    ]);

    let dialogue = Dialogue::load(
        PathBuf::from("./text/lab_1.json"),
        asset_server,
        character_map
    );
    

    let text_section = TextSection::new(
        "",
        TextStyle {
            // This font is loaded and will be used instead of the default font.
            font_size: 12.0,
            ..default()
        });

    let mut line_vector : Vec<TextSection> = vec![];
    for line in dialogue.lines {
        commands.spawn(line);
        line_vector.push(text_section.clone());

    }

    commands.spawn(
        (TextBundle::from_sections(
            line_vector
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Left)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        DialogueText{
            current_line_index : 0
        }));
}