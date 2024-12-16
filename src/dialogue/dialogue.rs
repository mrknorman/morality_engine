use std::{
	time::Duration, 
	path::PathBuf, 
	collections::HashMap, 
    fs::File, 
    io::BufReader
};
use bevy::{
    prelude::*, 
    sprite::Anchor, 
    text::TextBounds, 
    audio::Volume
};
use serde::{Serialize, Deserialize};

use crate::{
    audio::{continuous_audio, ContinuousAudioPallet}, 
    character::Character, 
    colors::PRIMARY_COLOR, 
    common_ui::NextButton, 
    game_states::{GameState, MainState, StateVector}, 
    graph::GraphPlugin, 
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
            .add_systems(Update, 
                activate_systems
            ).add_systems(
                Update,
                (
                    Dialogue::advance_dialogue,
                    Dialogue::skip_controls,
                    Dialogue::play
                ).run_if(in_state(DialogueSystemsActive::True))
            );

            if !app.is_plugin_added::<GraphPlugin>() {
                app.add_plugins(GraphPlugin);
            };
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

fn dialogue_text_bounds() -> TextBounds {
    TextBounds {
        width : Some(500.0), 
        height : Some(2000.0)
    }
}

fn dialogue_anchor() -> Anchor {
    Anchor::CenterLeft
}

#[derive(Component)]
#[require(Text2d, TextBounds(dialogue_text_bounds), Transform, Anchor(dialogue_anchor))]
pub struct Dialogue {
    pub lines: Vec<DialogueLine>,
    pub current_line_index: usize,
    pub current_char_index: usize,
    pub playing: bool,
    pub skip_count: usize,
    pub timer: Timer,
    pub char_duration_millis: u64,
    pub num_spans: usize
}

impl Dialogue {
    pub fn new(lines: Vec<DialogueLine>) -> Self {
        Dialogue {
            lines,
            current_line_index: 0,
            current_char_index: 0,
            playing: false,
            skip_count: 0,
            timer: Timer::new(
                Duration::from_secs_f32(0.0), 
                TimerMode::Repeating
            ),
            char_duration_millis: 50,
            num_spans : 1
        }
    }

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

        Self::new(lines)
    }

    pub fn init(
        dialogue_path: impl Into<PathBuf>,
        asset_server: &Res<AssetServer>,
        user_map: &HashMap<String, Character>
    ) -> (ContinuousAudioPallet, Dialogue) {
        let dialogue = Dialogue::load(dialogue_path.into(), user_map);

        let character_audio: Vec<(String, AudioPlayer::<AudioSource>, PlaybackSettings)> = dialogue.lines.iter()
            .map(|line| (
                line.character.name.clone(),
                AudioPlayer::<AudioSource>(asset_server.load(
                    line.character.audio_source_file_path.clone()
                )),
                PlaybackSettings{
                    paused : true,
                    volume : Volume::new(0.3),
                    ..continuous_audio()
                }
            ))
            .collect();

        (
            ContinuousAudioPallet::new(character_audio),
            dialogue
        )
    }

    fn generate_shell_prompt(
        username: &str,
        hostname: &str, 
        color : &Color
    ) -> Vec<(TextSpan, TextColor, TextFont)> {        
        vec![
            (
                TextSpan::new(
                    format!("{}@{}:\n    ", username, hostname)
                ),
                TextColor(*color),
                TextFont{
                    font_size : 12.0,
                    ..default()
                }
            ),
            (
                TextSpan::new(""),
                TextColor(PRIMARY_COLOR),
                TextFont{
                    font_size : 12.0,
                    ..default()
                }
            )
        ]
    }

	pub fn play(
        mut commands: Commands,
		mut query: Query<(Entity, &mut Dialogue, &mut ContinuousAudioPallet), With<Text2d>>,
		audio_query: Query<&AudioSink>,
		mut ev_advance_dialogue: EventReader<AdvanceDialogue>,
	) {
		for (
            entity,
            mut dialogue,
            audio_pallet
        ) in query.iter_mut() {
			if let Some(line) = dialogue.lines.get(
                dialogue.current_line_index
            ) {
				if !dialogue.playing && (
                    !ev_advance_dialogue.is_empty() || 
                    dialogue.current_line_index == 0
                ) {
					ev_advance_dialogue.clear();

                    commands.get_entity(entity).unwrap().with_children(
                        | parent | {
                            let shell_prompts = Self::generate_shell_prompt(
                                &line.character.name, 
                                &line.hostname, 
                                &line.character.color
                            );

                            for span in shell_prompts{
                                parent.spawn(
                                    span 
                                );
                            }         
                        }
                    );
                    dialogue.num_spans += 2;

					dialogue.playing = true;
					dialogue.timer = Timer::new(Duration::from_millis(
                        dialogue.char_duration_millis), 
                        TimerMode::Repeating
                    );
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
		mut query: Query<(Entity, &mut Dialogue, &mut ContinuousAudioPallet), With<Text2d>>,
		audio_query: Query<&AudioSink>,
        mut writer: Text2dWriter,
		keyboard_input: Res<ButtonInput<KeyCode>>,
	) {
		if !keyboard_input.just_pressed(KeyCode::Enter) {
			return;
		}
	
		for (
            entity,
            mut dialogue,
            audio_pallet
        ) in query.iter_mut() {

			if !dialogue.playing {
				continue;
			}
	
			if let Some(line) = dialogue.lines.get(
                dialogue.current_line_index
            ) {

				if let Some(&audio_entity) = audio_pallet.entities.get(
                    &line.character.name
                ) {
					if let Ok(audio_sink) = audio_query.get(audio_entity) {
						match dialogue.skip_count {
							0 => audio_sink.set_speed(4.0),
							_ => audio_sink.pause(),
						}
					}
				}
	
				if dialogue.skip_count == 0 {
					dialogue.skip_count += 1;
					dialogue.timer = Timer::new(
                        Duration::from_millis(
                            dialogue.char_duration_millis / 4
                        ), 
                        TimerMode::Repeating
                    );
				} else {
                    *writer.text(entity, dialogue.num_spans - 1) = line.raw_text.clone();
					dialogue.current_char_index = line.raw_text.len();
					dialogue.skip_count += 1;
				}
			}
		}
	}

	pub fn advance_dialogue(
        mut commands: Commands,
        mut query: Query<(Entity, &mut Dialogue, &mut ContinuousAudioPallet), With<Text2d>>,
        mut writer: Text2dWriter,
        audio_query: Query<&AudioSink>, 
        asset_server: Res<AssetServer>,
        windows: Query<&Window>,
        time: Res<Time>
    ) {
        for (entity, mut dialogue, audio_pallet) in query.iter_mut() {
            if !dialogue.playing || (!dialogue.timer.tick(time.delta()).finished() && dialogue.skip_count <= 1) {
                continue;
            }
    
            let line = &dialogue.lines[dialogue.current_line_index];
            let next_char = line.raw_text.chars().nth(dialogue.current_char_index);

            let span_index = dialogue.num_spans - 1;
    
            match next_char {
                Some(c) => {

                    writer.text(entity, span_index).push(c);
                    dialogue.current_char_index += 1;
                }
                None => {
                    Self::stop_audio_if_playing(&audio_pallet, &audio_query, &line.character.name);

                    writer.text(entity, span_index).push('\n');

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
                        entity,   // Pass the dialogue entity as the parent
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
                NextButton,
                TextButtonBundle::new(
                    asset_server,
                    actions,
                    vec![KeyCode::Enter],
                    format!("[{}]", instruction),
                    NextButton::translation(windows),
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