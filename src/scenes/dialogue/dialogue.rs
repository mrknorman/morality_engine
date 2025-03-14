use std::{
    time::Duration, 
    collections::HashMap
};

use bevy::{
    audio::Volume, 
    prelude::*, 
    sprite::Anchor, 
    text::TextBounds
};
use enum_map::{
    enum_map, 
    Enum
};
use serde::{
    Serialize,
    Deserialize
};

use crate::{
    data::{
        character::{
            Character, 
            CharacterKey
        },  
        states::GameState
    },
    systems::{
        audio::{
            continuous_audio, 
            ContinuousAudio, 
            ContinuousAudioPallet, 
            TransientAudio, 
            TransientAudioPallet
        }, 
        colors::PRIMARY_COLOR,
        interaction::{
            ActionPallet, 
            AdvanceDialogue, 
            Draggable, 
            InputAction
        },
        time::Dilation
    },
    entities::{
        graph::GraphPlugin,
        sprites::{
            window::WindowTitle, 
            SpritePlugin
        },
        text::{
            TextButton, 
            TextPlugin, 
            TextWindow
        }, 
    },
    style::common_ui::NextButton
};

use super::content::DialogueScene;

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
            if !app.is_plugin_added::<SpritePlugin>() {
                app.add_plugins(SpritePlugin);
            }
            if !app.is_plugin_added::<TextPlugin>() {
                app.add_plugins(TextPlugin);
            }

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

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueActions {
    AdvanceDialogue
}

impl std::fmt::Display for DialogueActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct DialogueLine {
    pub raw_text: String,
    pub hostname: String,
    pub instruction: String,
    pub character: Character,
}

fn dialogue_text_bounds() -> TextBounds {
    TextBounds {
        width : Some(400.0), 
        height : None
    }
}


fn dialogue_anchor() -> Anchor  {
    Anchor::CenterLeft
}

fn dialogue_window() -> TextWindow {
    TextWindow{
        title : Some(
            WindowTitle{
                text : String::from("User Prompt"),
                padding : 50.0
            }
        ),
        border_color : PRIMARY_COLOR,
        header_height : 25.0,
        padding : Vec2::new(50.0, 50.0),
        has_close_button : false
    }
}

#[derive(Component)]
#[require(Text2d, TextWindow(dialogue_window), Draggable, TextBounds(dialogue_text_bounds), Transform, Anchor(dialogue_anchor))]
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


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueSounds {
    Hum,
    Office,
    Music,
    Click
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

    pub fn load(content : &DialogueScene, user_map: &HashMap<String, Character>) -> Self {
        let loaded_dialogue: DialogueLoader = serde_json::from_str(content.content())
            .expect("Failed to parse embedded JSON");
    
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
        asset_server: &Res<AssetServer>,
        dialogue_content : &Vec<DialogueScene>,
        user_map: &HashMap<String, Character>
    ) -> (ContinuousAudioPallet<CharacterKey>, Dialogue) {

        let dialogue = dialogue_content
            .into_iter()
            .map(|file| Dialogue::load(file, user_map))
            .reduce(|mut acc, mut d| {
                acc.lines.extend(d.lines.drain(..));
                acc
            })
            .expect("At least one dialogue needed!");

        let character_audio: Vec<ContinuousAudio<CharacterKey>> = dialogue.lines.iter()
            .map(|line| 
            ContinuousAudio{   
                key : CharacterKey::Creator,
                source : AudioPlayer::<AudioSource>(asset_server.load(
                    line.character.audio_source_file_path.clone()
                )),
                settings : PlaybackSettings{
                    paused : true,
                    volume : Volume::new(0.5),
                    ..continuous_audio()
                },
                dilatable : true
            })
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

	fn play(
        mut commands: Commands,
		mut query: Query<(Entity, &mut Dialogue, &mut ContinuousAudioPallet<CharacterKey>), With<Text2d>>,
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
                    if let Ok(audio_sink) = audio_query.get(audio_pallet.entities[line.character.key]) {
                        audio_sink.play();
                    }
				}
			}
		}
	}

	fn skip_controls(
		mut query: Query<(Entity, &mut Dialogue, &mut ContinuousAudioPallet<CharacterKey>), With<Text2d>>,
		audio_query: Query<&AudioSink>,
        mut writer: Text2dWriter,
		keyboard_input: Res<ButtonInput<KeyCode>>,
        mouse_input : Res<ButtonInput<MouseButton>>,
        mut dilation : ResMut<Dilation>
	) {
		if !keyboard_input.just_pressed(KeyCode::Enter) && !mouse_input.just_pressed(MouseButton::Left)  {
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

				if dialogue.skip_count == 0 {
					dialogue.skip_count += 1;
					dilation.0 = 4.0;
				} else {

                    if let Ok(audio_sink) = audio_query.get(audio_pallet.entities[line.character.key]) {
                        audio_sink.pause();
                    }
                    if let Some(mut text) = writer.get_text(entity, dialogue.num_spans - 1) {
                        (*text).clone_from(&line.raw_text);

                        dialogue.current_char_index = line.raw_text.len();
                        dialogue.skip_count += 1;
                    };
				}
			}
		}
	}

	fn advance_dialogue(
        mut commands: Commands,
        mut query: Query<(Entity, &mut Dialogue, &mut ContinuousAudioPallet<CharacterKey>), With<Text2d>>,
        mut writer: Text2dWriter,
        audio_query: Query<&AudioSink>, 
        asset_server: Res<AssetServer>,
        time: Res<Time>,
        mut dilation : ResMut<Dilation>
    ) {
        for (entity, mut dialogue, audio_pallet) in query.iter_mut() {
            if !dialogue.playing || (!dialogue.timer.tick(time.delta().mul_f32(dilation.0)).finished() && dialogue.skip_count <= 1) {
                continue;
            }
    
            let line = &dialogue.lines[dialogue.current_line_index];
            let next_char = line.raw_text.chars().nth(dialogue.current_char_index);

            let span_index = dialogue.num_spans - 1;
    
            match next_char {
                Some(c) => {
                    if let Some(mut text) = writer.get_text(entity, span_index) {
                        (*text).push(c);
                    };
                    dialogue.current_char_index += 1;
                }
                None => {
                    Self::stop_audio_if_playing(&audio_pallet, &audio_query, line.character.key);
                    if let Some(mut text) = writer.get_text(entity, span_index) {
                        (*text).push('\n');
                    };

                    let next_action = if dialogue.current_line_index >= dialogue.lines.len() - 1 {
                        InputAction::NextScene
                    } else {
                        InputAction::AdvanceDialogue("placeholder".to_string())
                    };
    
                    Self::spawn_next_button(
                        &mut commands, 
                        entity,   // Pass the dialogue entity as the parent
                        &asset_server, 
                        next_action, 
                        &line.instruction
                    );

                    dilation.0 = 1.0;
                    dialogue.current_line_index += 1;
                    dialogue.current_char_index = 0;
                    dialogue.skip_count = 0;
                    dialogue.playing = false;
                }
            }
        }
    }
    
    fn stop_audio_if_playing(
        audio_pallet: &ContinuousAudioPallet<CharacterKey>,
        audio_query: &Query<&AudioSink>,
        character_key: CharacterKey
    ) {
        if let Ok(audio_sink) = audio_query.get(audio_pallet.entities[character_key]) {
            audio_sink.pause();
        }
    }
    
    fn spawn_next_button(
        commands: &mut Commands,
        dialogue_entity: Entity,  // Add parent entity parameter
        asset_server: &Res<AssetServer>,
        next_action: InputAction<DialogueSounds>,
        instruction: &str
    ) {

        commands.entity(dialogue_entity).with_children(|parent| {
            parent.spawn((
                NextButton,
                TextButton::new(
                    vec![DialogueActions::AdvanceDialogue],
                    vec![KeyCode::Enter],
                    format!("[{}]", instruction),
                ),
                ActionPallet::<DialogueActions, DialogueSounds>(
                    enum_map!(
                        DialogueActions::AdvanceDialogue => vec![
                            InputAction::PlaySound(DialogueSounds::Click),
                            next_action.clone(),
                            InputAction::Despawn(None)
                         ]
                     )
                ),
                TransientAudioPallet::new(
                    vec![(
                        DialogueSounds::Click,
                        vec![
                            TransientAudio::new(
                                asset_server.load("./audio/effects/mech_click.ogg"), 
                                0.1, 
                                true,
                                1.0,
                                true
                            )
                        ]
                    )]
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