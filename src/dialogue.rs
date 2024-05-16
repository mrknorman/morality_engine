
use std::{path::PathBuf, time::Duration, fs::File, io::BufReader, collections::HashMap};
use bevy::{asset::AssetServer, text::{BreakLineOn, Text2dBounds}, sprite::Anchor, prelude::*};
use serde::{Serialize, Deserialize};

use crate::audio::{play_sound_once, BackgroundAudio};
use crate::game_states::{MainState, GameState};

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
    skip_count : usize,
    timer: Timer,
    char_duration_milis : u64
} 

const LINE_WIDTH: usize = 50; // Adjust the width as needed
fn format_text(raw_text: &str) -> String {
    let mut formatted_text = String::new();
    let words: Vec<&str> = raw_text.split_whitespace().collect();
    let mut current_line: String = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > LINE_WIDTH - 2 {
            // Pad the current line to the desired width
            formatted_text.push('|');
            formatted_text.push_str(&format!("{:<1$}|", current_line, LINE_WIDTH - 2));
            formatted_text.push('\n');
            current_line.clear();
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    // Add the last line
    if !current_line.is_empty() {
        formatted_text.push('|');
        formatted_text.push_str(&format!("{:<1$}|", current_line, LINE_WIDTH - 2));
        formatted_text.push('\n');
    }

    formatted_text
}

impl DialogueLine {
    fn new(raw_text : &str, instructon_text : &str, index : usize) -> DialogueLine {
        DialogueLine {
            index,
            raw_text : String::from(raw_text),
            instructon_text : String::from(instructon_text),
            current_index : 0,
            playing : false,
            started : false,
            skip_count : 0,
            timer : Timer::new(Duration::from_millis(0), TimerMode::Repeating),
            char_duration_milis : 50
        }
    }
}

fn setup_line(
    text : &mut Mut<'_, Text>, 
    audio : Mut<'_, AudioSink>, 
    character : &Character,
    mut line : Mut<'_, DialogueLine>) {

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

pub fn play_dialogue (
        mut query_line: Query<(&Character, &mut DialogueLine, &mut AudioSink), Without<DialogueText>>,
        mut query_text: Query<(&mut Text, &DialogueText), Without<DialogueLine>>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut commands: Commands, 
        asset_server : Res<AssetServer>,
        mut next_main_state: ResMut<NextState<MainState>>,
        mut next_game_state: ResMut<NextState<GameState>>,

    ) {

    let (mut text, dialogue) = query_text.single_mut(); 

    if dialogue.current_line_index == 0 {
        for (character, line, audio) in query_line.iter_mut() {
            if line.index == 0 && !line.started {
                setup_line(&mut text, audio, character, line);
            }
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::Enter) {

        for (character, mut line, audio) in query_line.iter_mut() {
            if !line.started {
                if line.index == dialogue.current_line_index {

                    let _ = play_sound_once(
                        "sounds/mech_click.ogg", 
                        &mut commands, 
                        &asset_server
                    );

                    text.as_mut().sections[line.index].value += &character.name.as_str();
                    text.as_mut().sections[line.index].value += ":\n    ";
                
                    audio.play();
                    line.playing = true;
                    line.timer = Timer::new(
                        Duration::from_millis(line.char_duration_milis), 
                        TimerMode::Repeating
                    );
                    line.started = true;
                } 
            } else if line.playing && line.skip_count == 0 {
                line.skip_count += 1;
                line.timer = Timer::new(
                    Duration::from_millis(line.char_duration_milis / 4), 
                    TimerMode::Repeating
                );
                audio.set_speed(4.0);
            } else if line.playing && line.skip_count > 0 {
                text.as_mut().sections[line.index].value = character.name.clone();
                text.as_mut().sections[line.index].value += ":\n    ";
                text.as_mut().sections[line.index].value += line.raw_text.as_str();

                line.current_index = line.raw_text.len() + 1;
 
                line.skip_count += 1;
            } else if dialogue.current_line_index >= dialogue.total_num_lines {
                next_main_state.set(MainState::InGame);
                next_game_state.set(GameState::Dilemma);
            }
        }
    }
}

pub fn typewriter_effect(
        mut query_line: Query<(&mut DialogueLine,  &mut AudioSink)>,
        mut query_text: Query<(&mut Text, &mut DialogueText)>,
        time: Res<Time>
    ) {

    let (mut text, mut dialogue) = query_text.single_mut(); 
    
    // For each character in the dialogue, update all texts in the query
    for (mut line, audio) in query_line.iter_mut() {

        if line.playing {

            if (line.timer.tick(time.delta()).finished() || line.skip_count > 1) && line.index == dialogue.current_line_index {
                let char: Option<char> = line.raw_text.chars().nth(line.current_index);

                match char {
                    Some(_) => text.as_mut().sections[line.index].value.push(char.unwrap()), // Push character directly to each text,
                    None => {
                        audio.stop();
                        line.playing = false;
                        text.sections[line.index].value += "\n    [";
                        text.sections[line.index].value += line.instructon_text.as_str();
                        text.sections[line.index].value += "]\n";
                        dialogue.current_line_index += 1;
                    }
                }

                line.current_index += 1; 
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
            character,
            audio : AudioBundle {
                    source: asset_server.load(audio_source_file_path),
                    settings : PlaybackSettings {
                        paused : true,
                        mode:  bevy::audio::PlaybackMode::Loop,
                        ..default()
                    }
            },
            line
        }
    }
}

pub struct Dialogue {
	lines : Vec<DialogueLineBundle>
}

#[derive(Serialize, Deserialize, Debug)]
struct NextStateOptionLoader {
    key : String,
    state : GameState,
    requirements : usize
}

#[derive(Serialize, Deserialize, Debug)]
struct DialogueLineLoader {
    character: String,
    dialogue: String,
    instruction: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DialogueLoader {
    lines : Vec<DialogueLineLoader>,
    possible_exit_states : Vec<NextStateOptionLoader>
}

impl Dialogue {

	pub fn load(
            file_path: PathBuf,
            asset_server : &Res<AssetServer>,
            character_map : HashMap<String, Character>
        ) -> Dialogue {
        
        // Open the file in read-only mode with a buffered reader
        let file = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        
        let loaded_dialogue : DialogueLoader = serde_json::from_reader(reader).unwrap();

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
                    asset_server
                )
            );
        };

        Dialogue{
            lines : line_vec
        }
    }
}

#[derive(Component)]
pub struct ButtonClick;

#[derive(Component)]
pub struct DialogueText {
    current_line_index : usize,
    total_num_lines : usize
}

pub fn spawn_dialogue(
        mut commands: Commands, 
        asset_server : Res<AssetServer>
    ) {

    let character_map = HashMap::from([
        (String::from("The Creator"),  Character::new("The Creator", "sounds/typing.ogg"))
    ]);

    let dialogue = Dialogue::load(
        PathBuf::from("./text/lab_1.json"),
        &asset_server,
        character_map
    );

    let num_lines = dialogue.lines.len();

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

    commands.spawn(
        (Text2dBundle {
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
        }
    ));

    let hum_audio = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/hum.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.5),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

    let office_audio = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/office.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.5),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

    let background_audio: Vec<Entity> = vec![hum_audio, office_audio];

    commands.insert_resource(BackgroundAudio{audio: background_audio});

}

pub fn cleanup_dialogue(
        mut commands: Commands, 
        mut query_line: Query<(Entity, &Character, &mut DialogueLine, &mut AudioSink), Without<DialogueText>>,
        mut query_text: Query<(Entity, &mut Text, &DialogueText), Without<DialogueLine>>,
        background_audio : Res<BackgroundAudio>
    ) {

    for (entity, _, _, _) in query_line.iter_mut() {
        commands.entity(entity).despawn()
    }

    for (entity, _, _) in query_text.iter_mut() {
        commands.entity(entity).despawn()
    }

    for i in 0..background_audio.audio.len(){
        commands.entity(background_audio.audio[i]).despawn()
    }
}



