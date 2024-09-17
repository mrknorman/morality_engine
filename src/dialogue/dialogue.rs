use std::{
	time::Duration,
	path::PathBuf, 
	collections::HashMap,
	fs::File,
	io::BufReader
};
use bevy::{
	ecs::component::StorageType,
	asset::AssetServer, 
	prelude::*,
	text::{
        BreakLineOn, 
        Text2dBounds
    },
    sprite::Anchor
};
use serde::{
	Serialize, 
	Deserialize
};

use crate::{
	game_states::{
		MainState, 
		GameState,
		StateVector
	},
	character::Character,
	text::TextButtonBundle, 
    interaction::{InputAction, AdvanceDialogue},
    common_ui::NextButtonBundle
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DialogueSystemsActive {
    #[default]
    False,
    True
}

pub struct DialoguePlugin;
impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<DialogueSystemsActive>()
        .add_systems(
            Update,
            activate_systems
        ).add_systems(
            Update,
            (
				DialogueLine::first,
				DialogueLine::advance_dialogue,
				DialogueLine::skip_controls,
				DialogueLine::play
			)
            .run_if(
                in_state(DialogueSystemsActive::True)
            )
        );
    }
}

fn activate_systems(
	mut dialogue_state: ResMut<NextState<DialogueSystemsActive>>,
	dialogue_query: Query<&Dialogue>
) {
	if !dialogue_query.is_empty() {
		dialogue_state.set(DialogueSystemsActive::True)
	} else {
		dialogue_state.set(DialogueSystemsActive::False)
	}
}

#[derive(Component, Clone)]
pub struct DialogueLine {
    pub index : usize,
    pub raw_text : String,
	pub hostname : String,
    pub current_index : usize,
    pub playing : bool,
    pub started : bool,
    pub skip_count : usize,
    pub timer: Timer,
    pub char_duration_millis : u64,
	pub instruction : String
} 

#[derive(Component)]
pub struct DialogueText {
    pub current_line_index : usize,
    pub total_num_lines : usize
}

impl DialogueText {

	pub fn new(
		num_lines : u32
	) -> Self {

		DialogueText {
			current_line_index : 0,
			total_num_lines : num_lines as usize
		}
	}
}

impl DialogueLine {
    pub fn new(
		raw_text : String, 
		hostname : String,
		index : usize,
		instruction : String
	) -> DialogueLine {

        DialogueLine {
            index,
            raw_text,
			hostname,
            current_index : 0,
            playing : false,
            started : false,
            skip_count : 0,
            timer : Timer::new(
				Duration::from_secs_f32(0.0),
				TimerMode::Repeating
			),
            char_duration_millis : 50,
			instruction
        }
    }

	fn generate_shell_prompt(
		username : &str,
		hostname : &str
	) -> Vec<TextSection> {

		vec![
			TextSection::new(
				format!(
					"{}@{}:\n", 
					username,
					hostname
				),
				TextStyle {
					font_size: 15.0,
					..default()
				}
			),
			TextSection::new(
				"",
				TextStyle {
					font_size: 15.0,
					..default()
				}
			)
		]
	}

	fn start(
		mut line : Mut<'_, DialogueLine>,
		mut text : Mut<'_, Text>,
		audio : Mut<'_, AudioSink>,
		user : &Character
	) {
		text.as_mut().sections.append(
			&mut Self::generate_shell_prompt(
				user.name.as_str(), 
				line.hostname.as_str()
			)
		);
		audio.play();
		line.playing = true;
		line.timer = Timer::new(
			Duration::from_millis(line.char_duration_millis),
			TimerMode::Repeating,
		);
		line.started = true;
	}

	fn first(
		mut query_text: Query<&mut Text, With<DialogueText>>,
		mut query_line: Query<(
			&Character, &mut DialogueLine, &mut AudioSink
		)>,
	) {
		for (
			user, line, audio
		) in query_line.iter_mut() {
			
			if line.index == 0 && !line.started {
				if let Ok(mut text) = query_text.get_single_mut() {
					text.sections.clear();
					Self::start(
						line, 
						text, 
						audio, 
						user
					);
				}
			}
		}
	}
	
	pub fn play (
			mut query_line: Query<(
				&Character, 
				&mut DialogueLine, 
				&mut AudioSink
			), Without<DialogueText>>,
			mut query_text: Query<(
				&mut Text, 
				&DialogueText
			), Without<DialogueLine>>,
			mut ev_advance_dialogue: EventReader<AdvanceDialogue>,
		) {
		
		for (
			text, 
			dialogue
		) in query_text.iter_mut() { 

			if !ev_advance_dialogue.is_empty() {
				ev_advance_dialogue.clear();
		
				for (
					user, line, audio
				) in query_line.iter_mut() {
					// Check if the current line should be started
					if !line.started && line.index == dialogue.current_line_index {
						// Perform the start operation immediately
						Self::start(line, text, audio, user);
						break;  // We found the line to start, so exit the inner loop
					}
				}
			}

		}
	}

	pub fn skip_controls(
		mut query_line: Query<(
			&Character, 
			&mut DialogueLine, 
			&mut AudioSink
		), Without<DialogueText>>,
		mut query_text: Query<
			&mut Text, 
			(With<DialogueText>, 
			Without<DialogueLine>)>,
		keyboard_input: Res<ButtonInput<KeyCode>>
	) {

		for mut text in query_text.iter_mut() { 

			if keyboard_input.just_pressed(KeyCode::Enter) {

				for (
					_, 
					mut line, 
					audio
				) in query_line.iter_mut() {

					if line.playing && line.skip_count == 0 {
						line.skip_count += 1;
						line.timer = Timer::new(
							Duration::from_millis(line.char_duration_millis / 4), 
							TimerMode::Repeating
						);
						audio.set_speed(4.0);
					} else if line.playing && line.skip_count > 0 {
						text.as_mut().sections.pop();
						text.as_mut().sections.push(
							TextSection::new(
								line.raw_text.as_str(),
								TextStyle {
									font_size: 15.0,
									..default()
								}
							)
						);
						line.current_index = line.raw_text.len() + 1;
						line.skip_count += 1;
					}
				}
			}
		}
	}
	
	pub fn advance_dialogue(
			mut commands: Commands,
			mut query_line: Query<(&mut DialogueLine,  &mut AudioSink)>,
			mut query_text: Query<(&mut Text, &mut DialogueText)>,
			asset_server: Res<AssetServer>,
			windows: Query<&Window>,
			time: Res<Time>
		) {
	
		for (mut text, mut dialogue) in query_text.iter_mut() { 
		
			// For each user in the dialogue, update all texts in the query
			for (
				mut line, audio
			) in query_line.iter_mut() {
		
				if line.playing && (line.timer.tick(time.delta()).finished() || line.skip_count > 1) && line.index == dialogue.current_line_index {
					let char: Option<char> = line.raw_text.chars().nth(line.current_index);
		
					match char {
						Some(_) => match text.as_mut().sections.last_mut() {
							Some(section) => section.value.push(char.unwrap()),
							None => {}
						}, // Push user directly to each text,
						None => {
							audio.stop();
							line.playing = false;
							match text.as_mut().sections.last_mut() {
								Some(section) => section.value.push_str("\n"),
								None => {}
							}
							
							let next_action = if dialogue.current_line_index >= dialogue.total_num_lines - 1 {
								InputAction::ChangeState(StateVector::new(
									Some(MainState::InGame),
									Some(GameState::Dilemma),
									None,
								))
							} else {
								InputAction::AdvanceDialogue("placeholder".to_string())
							};
							
							let actions = vec![
								InputAction::PlaySound("click".to_string()),
								next_action,
								InputAction::Despawn,
							];
							
							commands.spawn((
								NextButtonBundle::new(),
								TextButtonBundle::new(
									&asset_server,
									actions,
									vec![KeyCode::Enter],
									format!("[{}]", line.instruction),
									NextButtonBundle::translation(&windows),
								),
							));							

							dialogue.current_line_index += 1;
						}
					}
		
					line.current_index += 1; 
				}
			}
		}
	}
}

#[derive(Bundle, Clone)]
pub struct DialogueLineBundle {
	user : Character,
	audio : AudioBundle,
	line : DialogueLine,
}

impl DialogueLineBundle {

	pub fn new(
		user : Character,
		line : DialogueLine,
		asset_server : &Res<AssetServer>
	) -> DialogueLineBundle {

		let audio_source_file_path = user.audio_source_file_path.clone();

		DialogueLineBundle {
			user,
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
    username: String,
	hostname: String,
    dialogue: String,
    instruction: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DialogueLoader {
    lines : Vec<DialogueLineLoader>,
    possible_exit_states : Vec<NextStateOptionLoader>
}

#[derive(Clone)]
pub struct Dialogue {
	pub lines : Vec<DialogueLineBundle>,
}

impl Dialogue {

	pub fn load(
            file_path: PathBuf,
            asset_server : &Res<AssetServer>,
            user_map : HashMap<String, Character>
        ) -> Dialogue {
        
        // Open the file in read-only mode with a buffered reader
        let file: File = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        
        let loaded_dialogue : DialogueLoader = serde_json::from_reader(reader).unwrap();

        let mut line_vec = vec![];
        for (index, line) in loaded_dialogue.lines.iter().enumerate() {

            let user_key = line.username.as_str();
            let user  = user_map.get(user_key)
                .expect(format!(
					"Character {user_key} not found in map on line {index}"
				).as_str()) // This replaces unwrap() for better error handling
                .clone(); // This clones the Character object

            line_vec.push(
                DialogueLineBundle::new(
                    user,
                    DialogueLine::new(
                        line.dialogue.clone(), 
						line.hostname.clone(),
                        index,
						line.instruction.clone()
                    ),
                    asset_server
                )
            );
        };

        Dialogue{
            lines : line_vec,
        }
    }
}

#[derive(Bundle)]
pub struct DialogueBundle{
	marker : DialogueText,
	dialogue : Dialogue,
	text : Text2dBundle
}

impl DialogueBundle {

	pub fn load(
		dialogue_path : impl Into<PathBuf>,
		asset_server : &Res<AssetServer>,
		user_map : HashMap<String, Character>
	) -> Self {

		let dialogue = Dialogue::load(
			dialogue_path.into(),
			&asset_server,
			user_map
		);
		let num_lines: u32 = dialogue.lines.len() as u32;	

		let line_vector: Vec<TextSection> = dialogue.lines.iter().map(|_| {
			TextSection::new(
				"",
				TextStyle {
					font_size: 15.0,
					..default()
				}
			)
		}).collect();

		let text = Text2dBundle {
			text : Text {
				sections : line_vector, //vec![],
				justify : JustifyText::Left, 
				linebreak_behavior: BreakLineOn::WordBoundary
			},
			text_2d_bounds : Text2dBounds {
				size: Vec2::new(500.0, 2000.0),
			},
			transform: Transform::from_xyz(-500.0,0.0, 1.0),
			text_anchor : Anchor::CenterLeft,
			..default()
		};

		DialogueBundle{
			marker : DialogueText::new(num_lines),
			dialogue,
			text
		}
	}
}

impl Component for Dialogue {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {

				let dialogue: Option<Dialogue> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<Dialogue>()
                        .map(|dialouge: &Dialogue| dialouge.clone())
                };

				// Step 2: Spawn child entities and collect their IDs
				let mut commands = world.commands();

				if let Some(dialogue) = dialogue {
                    commands.entity(entity).with_children(
						|parent| {
                        for line in dialogue.lines {
							parent.spawn(line);
						}
                    });
				}
            }
        );
    }
}