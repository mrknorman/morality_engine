use std::{
	time::Duration,
	path::PathBuf, 
	collections::HashMap,
	fs::File,
	io::BufReader
};

use bevy::{
	asset::AssetServer, 
	prelude::*
};
use serde::{
	Serialize, 
	Deserialize
};

use crate::{
	audio::play_sound_once,
	game_states::{
		MainState, 
		GameState
	},
	character::Character
};

#[derive(Component)]
pub struct DialogueLine {
    pub index : usize,
    pub raw_text : String,
    pub instructon_text : String,
    pub current_index : usize,
    pub playing : bool,
    pub started : bool,
    pub skip_count : usize,
    pub timer: Timer,
    pub char_duration_milis : u64
} 

#[derive(Component)]
pub struct DialogueText {
    pub current_line_index : usize,
    pub total_num_lines : usize
}

impl DialogueLine {
    pub fn new(
		raw_text : &str, 
		instructon_text : &str, 
		index : usize
	) -> DialogueLine {

        DialogueLine {
            index,
            raw_text : String::from(raw_text),
            instructon_text : String::from(instructon_text),
            current_index : 0,
            playing : false,
            started : false,
            skip_count : 0,
            timer : Timer::new(
				Duration::from_secs_f32(0.0),
				TimerMode::Repeating
			),
            char_duration_milis : 50
        }
    }

	fn setup(
		text : &mut Mut<'_, Text>, 
		audio : Mut<'_, AudioSink>, 
		character : &Character,
		mut line : Mut<'_, DialogueLine>
	) {
	
		text.as_mut().sections[line.index].value += character.name.as_str();
		text.as_mut().sections[line.index].value += "@pan_box: ";
	
		audio.play();
		line.playing = true;
		line.timer = Timer::new(
			Duration::from_millis(line.char_duration_milis), 
			TimerMode::Repeating
		);
		line.started = true;
	}
	
	pub fn play (
			mut query_line: Query<(
				&Character, &mut DialogueLine, &mut AudioSink
			), Without<DialogueText>>,
			mut query_text: Query<(&mut Text, &DialogueText), Without<DialogueLine>>,
			keyboard_input: Res<ButtonInput<KeyCode>>,
			mut commands: Commands, 
			asset_server : Res<AssetServer>,
			mut next_main_state: ResMut<NextState<MainState>>,
			mut next_game_state: ResMut<NextState<GameState>>
		) {
	
		let (mut text, dialogue) = query_text.single_mut(); 
	
		if dialogue.current_line_index == 0 {
			for (
				character, 
				line, 
				audio) in query_line.iter_mut() {
				if line.index == 0 && !line.started {
					Self::setup(&mut text, audio, character, line);
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
	
						text.as_mut().sections[line.index].value += character.name.as_str();
						text.as_mut().sections[line.index].value += "@pan_box: ";
					
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
					text.as_mut().sections[line.index].value.clone_from(&character.name);
					text.as_mut().sections[line.index].value += "@pan_box: ";
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
	
			if line.playing && (line.timer.tick(time.delta()).finished() || line.skip_count > 1) && line.index == dialogue.current_line_index {
				let char: Option<char> = line.raw_text.chars().nth(line.current_index);
	
				match char {
					Some(_) => text.as_mut().sections[line.index].value.push(char.unwrap()), // Push character directly to each text,
					None => {
						audio.stop();
						line.playing = false;
						text.sections[line.index].value += "\n";
						//text.sections[line.index].value += line.instructon_text.as_str();
						//text.sections[line.index].value += "]\n\n";
						dialogue.current_line_index += 1;
					}
				}
	
				line.current_index += 1; 
			}
		}
	}
}

#[derive(Bundle)]
pub struct DialogueLineBundle {
	character : Character,
	audio : AudioBundle,
	line : DialogueLine,
}

impl DialogueLineBundle {

	pub fn new(
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

#[derive(Component)]
pub struct Dialogue {
	pub lines : Vec<DialogueLineBundle>
}

impl Dialogue {

	pub fn load(
            file_path: PathBuf,
            asset_server : &Res<AssetServer>,
            character_map : HashMap<String, Character>
        ) -> Dialogue {
        
        // Open the file in read-only mode with a buffered reader
        let file: File = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        
        let loaded_dialogue : DialogueLoader = serde_json::from_reader(reader).unwrap();

        let mut line_vec = vec![];
        for (index, line) in loaded_dialogue.lines.iter().enumerate() {

            let character_key = line.character.as_str();
            let character  = character_map.get(character_key)
                .expect(format!("Character {character_key} not found in map on line {index}").as_str()) // This replaces unwrap() for better error handling
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

#[derive(Bundle)]
pub struct DialogueBundle{
	dialogue : Dialogue,
	marker : DialogueText
}

impl DialogueBundle {

	//pub fn load() -> Self {
	//
	//
	//}
}