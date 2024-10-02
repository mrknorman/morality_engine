use std::{
	time::Duration, 
	path::PathBuf, 
	collections::HashMap, 
    fs::File, 
    io::BufReader
};
use bevy::{prelude::*, text::{BreakLineOn, Text2dBounds}, sprite::Anchor};
use serde::{Serialize, Deserialize};

use crate::{
    audio::{ContinuousAudioPallet, ContinuousAudio},
    character::Character,
    common_ui::NextButtonBundle,
    game_states::{GameState, MainState, StateVector},
    interaction::{AdvanceDialogue, InputAction},
    text::TextButtonBundle
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
            .add_systems(Update, activate_systems)
            .add_systems(
                Update,
                (
                    Dialogue::advance_dialogue,
                    Dialogue::skip_controls,
                    Dialogue::play
                ).run_if(in_state(DialogueSystemsActive::True))
            );
    }
}

fn activate_systems(
    mut dialogue_state: ResMut<NextState<DialogueSystemsActive>>,
    dialogue_query: Query<&Dialogue>
) {
    dialogue_state.set(if dialogue_query.is_empty() {
        DialogueSystemsActive::False
    } else {
        DialogueSystemsActive::True
    });
}

#[derive(Clone)]
pub struct DialogueLine {
    pub raw_text: String,
    pub hostname: String,
    pub instruction: String,
    pub character: Character,
}

#[derive(Component)]
pub struct Dialogue {
    pub lines: Vec<DialogueLine>,
    pub current_line_index: usize,
    pub current_char_index: usize,
    pub playing: bool,
    pub skip_count: usize,
    pub timer: Timer,
    pub char_duration_millis: u64,
}

impl Dialogue {
    pub fn new(lines: Vec<DialogueLine>) -> Self {
        Dialogue {
            lines,
            current_line_index: 0,
            current_char_index: 0,
            playing: false,
            skip_count: 0,
            timer: Timer::new(Duration::from_secs_f32(0.0), TimerMode::Repeating),
            char_duration_millis: 50,
        }
    }

    fn generate_shell_prompt(
        username: &str,
        hostname: &str, 
        color : &Color
    ) -> Vec<TextSection> {        
        vec![
            TextSection::new(
                format!("{}@{}:\n    ", username, hostname),
                TextStyle { 
                    font_size: 15.0,
                    color : *color, 
                    ..default()
                }
            ),
            TextSection::new("", TextStyle { font_size: 15.0, ..default() })
        ]
    }

	pub fn play(
		mut query: Query<(&mut Dialogue, &mut Text, &mut ContinuousAudioPallet)>,
		audio_query: Query<&AudioSink>,
		mut ev_advance_dialogue: EventReader<AdvanceDialogue>,
	) {
		for (mut dialogue, mut text, audio_pallet) in query.iter_mut() {
			if let Some(line) = dialogue.lines.get(dialogue.current_line_index) {
				if !dialogue.playing && (!ev_advance_dialogue.is_empty() || dialogue.current_line_index == 0) {
					ev_advance_dialogue.clear();
					text.sections.extend(Self::generate_shell_prompt(
                        &line.character.name, 
                        &line.hostname, 
                        &line.character.color
                    ));
					dialogue.playing = true;
					dialogue.timer = Timer::new(Duration::from_millis(dialogue.char_duration_millis), TimerMode::Repeating);
				} else if dialogue.playing {
					if let Some(&audio_entity) = audio_pallet.entities.get(&line.character.name) {
						if let Ok(audio_sink) = audio_query.get(audio_entity) {
							audio_sink.play();
						}
					} else {
						error!("Audio entity not found for {}", &line.character.name);
					}
				}
			}
		}
	}

	pub fn skip_controls(
		mut query: Query<(&mut Dialogue, &mut Text, &mut ContinuousAudioPallet)>,
		audio_query: Query<&AudioSink>,
		keyboard_input: Res<ButtonInput<KeyCode>>,
	) {
		if !keyboard_input.just_pressed(KeyCode::Enter) {
			return;
		}
	
		for (mut dialogue, mut text, audio_pallet) in query.iter_mut() {
			if !dialogue.playing {
				continue;
			}
	
			if let Some(line) = dialogue.lines.get(dialogue.current_line_index) {
				if let Some(&audio_entity) = audio_pallet.entities.get(&line.character.name) {
					if let Ok(audio_sink) = audio_query.get(audio_entity) {
						match dialogue.skip_count {
							0 => audio_sink.set_speed(4.0),
							_ => audio_sink.pause(),
						}
					}
				}
	
				if dialogue.skip_count == 0 {
					dialogue.skip_count += 1;
					dialogue.timer = Timer::new(Duration::from_millis(dialogue.char_duration_millis / 4), TimerMode::Repeating);
				} else {
					text.sections.pop();
					text.sections.push(TextSection::new(
						&line.raw_text,
						TextStyle { font_size: 15.0, ..default() }
					));
					dialogue.current_char_index = line.raw_text.len();
					dialogue.skip_count += 1;
				}
			}
		}
	}

	pub fn advance_dialogue(
        mut commands: Commands,
        mut query: Query<(Entity, &mut Dialogue, &mut Text, &mut ContinuousAudioPallet)>,
        audio_query: Query<&AudioSink>,  // Query for AudioSink components
        asset_server: Res<AssetServer>,
        windows: Query<&Window>,
        time: Res<Time>
    ) {
        for (dialogue_entity, mut dialogue, mut text, audio_pallet) in query.iter_mut() {
            if !dialogue.playing || (!dialogue.timer.tick(time.delta()).finished() && dialogue.skip_count <= 1) {
                continue;
            }
    
            let line = &dialogue.lines[dialogue.current_line_index];
            let next_char = line.raw_text.chars().nth(dialogue.current_char_index);
    
            match next_char {
                Some(c) => {
                    if let Some(section) = text.sections.last_mut() {
                        section.value.push(c);
                    }
                    dialogue.current_char_index += 1;
                }
                None => {
                    Self::stop_audio_if_playing(&audio_pallet, &audio_query, &line.character.name);
    
                    if let Some(section) = text.sections.last_mut() {
                        section.value.push('\n');
                    }
    
                    let next_action = if dialogue.current_line_index >= dialogue.lines.len() - 1 {
                        InputAction::ChangeState(StateVector::new(
                            Some(MainState::InGame),
                            Some(GameState::Dilemma),
                            None,
                        ))
                    } else {
                        InputAction::AdvanceDialogue("placeholder".to_string())
                    };
    
                    Self::spawn_next_button(
                        &mut commands, 
                        dialogue_entity,   // Pass the dialogue entity as the parent
                        &asset_server, 
                        &windows, 
                        next_action, 
                        &line.instruction
                    );
    
                    dialogue.current_line_index += 1;
                    dialogue.current_char_index = 0;
                    dialogue.skip_count = 0;
                    dialogue.playing = false;
                }
            }
        }
    }
    
    fn stop_audio_if_playing(
        audio_pallet: &ContinuousAudioPallet,
        audio_query: &Query<&AudioSink>,
        character_name: &str
    ) {
        if let Some(&audio_entity) = audio_pallet.entities.get(character_name) {
            if let Ok(audio_sink) = audio_query.get(audio_entity) {
                audio_sink.pause();
                audio_sink.set_speed(1.0); // Reset the audio playback speed
            }
        }
    }
    
    fn spawn_next_button(
        commands: &mut Commands,
        dialogue_entity: Entity,  // Add parent entity parameter
        asset_server: &Res<AssetServer>,
        windows: &Query<&Window>,
        next_action: InputAction,
        instruction: &str
    ) {
        let actions = vec![
            InputAction::PlaySound("click".to_string()),
            next_action,
            InputAction::Despawn,
        ];
    
        commands.entity(dialogue_entity).with_children(|parent| {
            parent.spawn((
                NextButtonBundle::new(),
                TextButtonBundle::new(
                    asset_server,
                    actions,
                    vec![KeyCode::Enter],
                    format!("[{}]", instruction),
                    NextButtonBundle::translation(windows),
                ),
            ));
        });
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct NextStateOptionLoader {
    key: String,
    state: GameState,
    requirements: usize
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
    lines: Vec<DialogueLineLoader>,
    possible_exit_states: Vec<NextStateOptionLoader>
}

impl Dialogue {
    pub fn load(file_path: PathBuf, user_map: &HashMap<String, Character>) -> Self {
        let file = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        
        let loaded_dialogue: DialogueLoader = serde_json::from_reader(reader).unwrap();

        let lines = loaded_dialogue.lines.into_iter()
            .map(|line| {
                let character = user_map.get(&line.username)
                    .expect(&format!("Character {} not found in user map", line.username))
                    .clone();
                DialogueLine {
                    raw_text: line.dialogue,
                    hostname: line.hostname,
                    instruction: line.instruction,
                    character,
                }
            })
            .collect();

        Dialogue::new(lines)
    }
}

#[derive(Bundle)]
pub struct DialogueBundle {
    audio: ContinuousAudioPallet,
    dialogue: Dialogue,
    text: Text2dBundle
}

impl DialogueBundle {
    pub fn load(
        dialogue_path: impl Into<PathBuf>,
        asset_server: &Res<AssetServer>,
        user_map: &HashMap<String, Character>
    ) -> Self {
        let dialogue = Dialogue::load(dialogue_path.into(), user_map);

        let text = Text2dBundle {
            text: Text {
                sections: vec![],
                justify: JustifyText::Left,
                linebreak_behavior: BreakLineOn::WordBoundary
            },
            text_2d_bounds: Text2dBounds {
                size: Vec2::new(500.0, 2000.0),
            },
            transform: Transform::from_xyz(-500.0, 0.0, 1.0),
            text_anchor: Anchor::CenterLeft,
            ..default()
        };

        let character_audio: Vec<(String, ContinuousAudio)> = dialogue.lines.iter()
            .map(|line| (
                line.character.name.clone(),
                ContinuousAudio::new(
                    asset_server,
                    line.character.audio_source_file_path.clone(),
                    0.3,
					true
                )
            ))
            .collect();

        DialogueBundle {
            audio: ContinuousAudioPallet::new(character_audio),
            dialogue,
            text
        }
    }
}