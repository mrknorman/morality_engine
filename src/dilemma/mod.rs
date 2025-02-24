use std::{
	path::PathBuf,
	time::Duration
};
use bevy::{
	prelude::*,
	audio::Volume,
	sprite::Anchor
};
use enum_map::{Enum, enum_map};
use crate::{
	ascii_fonts::{
		AsciiPlugin, 
		AsciiString
	}, audio::{
		continuous_audio, one_shot_audio, ContinuousAudio, ContinuousAudioPallet, DilatableAudio, MusicAudio, NarrationAudio, OneShotAudio, OneShotAudioPallet, TransientAudio, TransientAudioPallet
	}, background::{
		Background, 
		BackgroundPlugin, 
		BackgroundSystems
	}, colors::{
		ColorAnchor, 
		ColorChangeEvent, 
		ColorChangeOn, 
		ColorTranslation, 
		Fade,
		BACKGROUND_COLOR, 
		DANGER_COLOR,
		DIM_BACKGROUND_COLOR, 
		OPTION_1_COLOR, 
		OPTION_2_COLOR, 
		PRIMARY_COLOR
	}, common_ui::{
		CenterLever, 
		DilemmaTimerPosition, 
		NextButton
	}, game_states::{
		DilemmaPhase, 
		GameState, 
		MainState, 
		StateVector
	}, 
	inheritance::BequeathTextColor, 
	interaction::{
		ActionPallet, Clickable, ClickablePong, InputAction, InteractionPlugin, InteractionState, KeyMapping, Pressable
	}, io::IOPlugin, motion::{
		Bounce, 
		PointToPointTranslation,
	}, 
	person::PersonPlugin, 
	physics::Velocity, 
	text::TextButton, 
	time::DilationTranslation, 
	timing::{
        TimerConfig, 
		TimerPallet, 
		TimerStartCondition,
		TimingPlugin
    }, 
	train::{
        Train, 
		TrainPlugin, 
		STEAM_TRAIN
    }
};

mod dilemma;
use dilemma::{
	cleanup_decision,
	Dilemma, 
	DilemmaPlugin, 
	DilemmaTimer
};
pub mod lever;
use lever::{
	Lever, 
	LeverPlugin, 
	LeverState, 
	LEVER_LEFT, 
	LEVER_MIDDLE, 
	LEVER_RIGHT
};
mod junction;
use junction::{
	Junction, 
	JunctionPlugin
};

pub struct DilemmaScreenPlugin;
impl Plugin for DilemmaScreenPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(GameState::Dilemma), 
			setup_dilemma
		)
		.add_systems(
			OnEnter(DilemmaPhase::Intro), 
			setup_dilemma_intro
			.run_if(in_state(GameState::Dilemma))
		)
		.add_systems(
			Update,
			spawn_delayed_children
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::Intro))
		).add_systems(
			OnEnter(DilemmaPhase::IntroDecisionTransition), 
			setup_dilemma_transition
		)
		.add_systems(
			Update,
			end_dilemma_transition
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::IntroDecisionTransition)),
		).add_systems(
			OnEnter(DilemmaPhase::Decision),
			setup_decision
			.run_if(in_state(GameState::Dilemma)),
		)
		.add_systems(
			OnExit(DilemmaPhase::Decision), 
			cleanup_decision
		)
		.add_systems(
			OnEnter(DilemmaPhase::ConsequenceAnimation),
			setup_dilemma_consequence_animation,
		)
		.add_systems(
			Update,
			spawn_delayed_children_consequence
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::ConsequenceAnimation)),
		);

		if !app.is_plugin_added::<TrainPlugin>() {
			app.add_plugins(TrainPlugin);
		}
		if !app.is_plugin_added::<AsciiPlugin>() {
			app.add_plugins(AsciiPlugin);
		}
		if !app.is_plugin_added::<LeverPlugin>() {
			app.add_plugins(LeverPlugin);
		}
		if !app.is_plugin_added::<PersonPlugin>() {
			app.add_plugins(PersonPlugin);
		}
		if !app.is_plugin_added::<InteractionPlugin>() {
			app.add_plugins(InteractionPlugin);
		}
		if !app.is_plugin_added::<TimingPlugin>() {
			app.add_plugins(TimingPlugin);
		}
		if !app.is_plugin_added::<BackgroundPlugin>() {
			app.add_plugins(BackgroundPlugin);
		}
		if !app.is_plugin_added::<JunctionPlugin>() {
			app.add_plugins(JunctionPlugin);
		}
		if !app.is_plugin_added::<DilemmaPlugin>() {
			app.add_plugins(DilemmaPlugin);
		}
        if !app.is_plugin_added::<IOPlugin>() {
            app.add_plugins(IOPlugin);
        }
    }
}


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaActions {
    StartDilemma
}

impl std::fmt::Display for DilemmaActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component)]
struct DilemmaRoot;
pub fn setup_dilemma(
		mut commands : Commands,
		asset_server: Res<AssetServer>
	) {

	let dilemma : Dilemma = Dilemma::load(
		PathBuf::from("./dilemmas/lab_1.json")
	);
	
	commands.spawn(
		(
			DilemmaRoot,
			StateScoped(GameState::Dilemma),
			Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
			Visibility::default()
		)
	).with_children(
        |parent: &mut ChildBuilder<'_>| {
			parent.spawn((
                MusicAudio,
				AudioPlayer::<AudioSource>(asset_server.load(
					"./music/algorithm_of_fate.ogg"
				)),
				PlaybackSettings{
					paused : false,
					volume : Volume::new(0.3),
					..continuous_audio()
				}
            ));

			let speed: f32 = -1000.0;
			let decision_position = -70.0 * dilemma.countdown_duration.as_secs_f32();
			let transition_duration = Duration::from_secs_f32(decision_position/speed);
			let train_initial_position = Vec3::new(120.0, -10.0, 1.0);
			let train_x_displacement = Vec3::new(decision_position, 0.0, 0.0);

			parent.spawn((
				TextColor(PRIMARY_COLOR),
				AsciiString(format!("DILEMMA {}", dilemma.index)),
				Fade(transition_duration),
				Transform::from_xyz(-400.0,300.0, 1.0)
			));

			parent.spawn((
				TextColor(PRIMARY_COLOR),
				Fade(transition_duration),
				Text2d::new(&dilemma.name),
				TextFont{
					font_size : 60.0,
					..default()
				},
				TextLayout {
					justify : JustifyText::Left,
					linebreak  : LineBreak::WordBoundary
				},
				Anchor::TopCenter,
				Transform::from_xyz(0.0,250.0, 1.0)
			));	
			
			parent.spawn((
				TextColor(BACKGROUND_COLOR),
				Background::load_from_json(
					"text/backgrounds/desert.json",	
					0.000002,
					-0.5
				),
				BequeathTextColor,
				ColorTranslation::new( //something to do with this???
					DIM_BACKGROUND_COLOR,
					transition_duration
				))
			);

			parent.spawn((
				Train::init(
					&asset_server,
					STEAM_TRAIN,
					0.0
				),
				Transform::from_translation(train_initial_position), 
				PointToPointTranslation::new(
					train_initial_position + train_x_displacement,
					transition_duration
				)
			));

			let final_position = Vec3::new(
				150.0 * dilemma.countdown_duration.as_secs_f32(),
				0.0, 
				0.0
			);

			let main_track_translation_end: Vec3 = Vec3::new(0.0, -40.0, 0.0);
			let main_track_translation_start: Vec3 = main_track_translation_end + final_position;
			let track_colors = vec![OPTION_1_COLOR, OPTION_2_COLOR];
			let initial_color = match dilemma.default_option {
				None => Color::WHITE,
				Some(ref option) => track_colors[*option]
			};

			parent.spawn((
				Junction{
					dilemma : dilemma.clone()
				},
				TextColor(BACKGROUND_COLOR),
				ColorTranslation::new(
					initial_color,
					transition_duration
				),
				Transform::from_translation(main_track_translation_start),
				PointToPointTranslation::new(
					main_track_translation_end,
					transition_duration
				)
			));
		}
	);

	commands.insert_resource(dilemma);
}

#[derive(Component)]
struct DilemmaIntroRoot;

pub fn setup_dilemma_intro(
	mut commands : Commands
	) {

	commands.spawn(
		(
			DilemmaIntroRoot,
			StateScoped(DilemmaPhase::Intro),
			Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
			Visibility::default(),
			TimerPallet::new(
				vec![
					(
						"narration".to_string(),
						TimerConfig::new(
							TimerStartCondition::Immediate, 
							1.0,
							None
						)
					),
					(
						"button".to_string(),
						TimerConfig::new(
							TimerStartCondition::Immediate, 
							2.0,
							None
						)
					)
				]
			)
		)
	);
}

fn spawn_delayed_children(
    mut commands: Commands,
    loading_query: Query<(Entity, &TimerPallet), With<DilemmaIntroRoot>>,
    asset_server: Res<AssetServer>
) {
    for (entity, timers) in loading_query.iter() {

		if let Some(narration_timer) = timers.timers.get(
            "narration"
        ) {
            if narration_timer.just_finished() {
				commands.entity(entity).with_children(
                    |parent| {
                    parent.spawn((
                        NarrationAudio,
						AudioPlayer::<AudioSource>(asset_server.load(
							"sounds/dilemma_narration/lab_1.ogg",
						)),
						PlaybackSettings{
							paused : false,
							volume : Volume::new(1.0),
							..one_shot_audio()
						}
                    ));
                });

            }
        } else {
            warn!("Entity {:?} is missing the 'button' timer", entity);
        }

        // Handle narration timer
        if let Some(button_timer) = timers.timers.get(
            "button"
        ) {
            if button_timer.just_finished() {                
				commands.entity(entity).with_children(|parent| {
					parent.spawn((
						NextButton,
						TextButton::new(
							vec![DilemmaActions::StartDilemma],
							vec![KeyCode::Enter],
							"[ Click here or Press Enter to Test Your Morality ]",
						),
						ActionPallet::<DilemmaActions, DilemmaSounds>(
							enum_map!(
								DilemmaActions::StartDilemma => vec![
									InputAction::PlaySound(DilemmaSounds::Click),
									InputAction::ChangeState(
										StateVector::new(
											Some(MainState::InGame),
											Some(GameState::Dilemma),
											Some(DilemmaPhase::IntroDecisionTransition),
										)
									),
									InputAction::Despawn
								 ]
							 )
						),
						TransientAudioPallet::new(
							vec![(
								DilemmaSounds::Click,
								vec![
									TransientAudio::new(
										asset_server.load("sounds/mech_click.ogg"), 
										0.1, 
										true,
										1.0
									)
								],
								Some(DilatableAudio)
							)]
						)
					));
				});
            }
        } else {
            warn!("Entity {:?} is missing the 'button' timer", entity);
        }
    }
}

pub fn setup_dilemma_transition(
		dilemma : Res<Dilemma>,
		mut commands : Commands,
		systems: Res<BackgroundSystems>,
		mut background_query : Query<(&mut Background, &mut ColorTranslation)>,
		mut train_query : Query<&mut PointToPointTranslation, (With<Train>, Without<Junction>)>,
		mut junction_query: Query<&mut PointToPointTranslation, (With<Junction>, Without<Train>)>,
		mut title_query: Query<(Entity, &mut ColorTranslation), Without<Background>>
	) {
	for mut train in train_query.iter_mut() {
		train.start();
	}
	for mut junction in junction_query.iter_mut() {
		junction.start()
	}
	for (entity, mut color) in title_query.iter_mut() {
		commands.entity(entity).insert(BequeathTextColor);
		commands.entity(entity).remove::<Bounce>();

		color.start()
	}
	for (mut background, mut color) in background_query.iter_mut() {
		color.start();
		background.speed = -dilemma.countdown_duration.as_secs_f32() / 5.0;
		commands.run_system(systems.0["update_background_speeds"]);
	}
}

pub fn end_dilemma_transition(
		dilemma: Res<Dilemma>,
		mut commands : Commands,
		systems: Res<BackgroundSystems>,
		mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
		mut train_query : Query<&mut PointToPointTranslation, (With<Train>, Without<Junction>)>,
		mut junction_query: Query<&mut PointToPointTranslation, (With<Junction>, Without<Train>)>,
		mut background_query : Query<&mut Background>
	) {
	
	let mut all_translations_finished = true;
	for translation in train_query.iter_mut() {
		all_translations_finished &= translation.timer.finished();
	}
	for translation in junction_query.iter_mut() {
		all_translations_finished &= translation.timer.finished();
	}
	
	if all_translations_finished {

		for mut background in background_query.iter_mut() {
			background.speed = 0.0;
			commands.run_system(systems.0["update_background_speeds"]);
		}
		
		next_sub_state.set(
			DilemmaPhase::Decision
		);

		for mut translation in train_query.iter_mut() {
			let initial_position = translation.initial_position;
			translation.initial_position = translation.final_position;
			translation.final_position = initial_position + Vec3::new(-100.0, 0.0, 0.0);
			translation.timer = Timer::new(
				dilemma.countdown_duration,
				TimerMode::Once
			);
		}
	}
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct DecisionRoot;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeverActions {
	LeftPull,
	RightPull
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaSounds {
	TrainAproaching,
	Clock,
	Click,
	Lever
}

pub fn setup_decision(
		mut commands : Commands,
		asset_server: Res<AssetServer>,
		dilemma: Res<Dilemma>,
	) {

	let (start_text, state, color) = match dilemma.default_option {
		None => (LEVER_MIDDLE, LeverState::Random, Color::WHITE),
		Some(ref option) if *option == 0 => (LEVER_LEFT, LeverState::Left, OPTION_1_COLOR),
		Some(_) => (LEVER_RIGHT, LeverState::Right, OPTION_2_COLOR),
	};

	commands.insert_resource(Lever(state));

	commands.spawn((
		StateScoped(DilemmaPhase::Decision),
		DecisionRoot
	)).with_children(
        |parent| {
            parent.spawn(
                ContinuousAudioPallet::new(
                    vec![
						ContinuousAudio{
							key : DilemmaSounds::TrainAproaching,
							source : AudioPlayer::<AudioSource>(asset_server.load(
								"./sounds/train_aproaching.ogg"
							)),
							settings : PlaybackSettings{
								volume : Volume::new(1.0),
								..continuous_audio()
							},
							dilatable : true 
						},
						ContinuousAudio{
							key : DilemmaSounds::Clock,
							source : AudioPlayer::<AudioSource>(asset_server.load(
								"./sounds/clock.ogg"
							)),
							settings : PlaybackSettings{
								volume : Volume::new(0.3),
								..continuous_audio()
							},
							dilatable : true 
						}
                    ]
                )
            );

			parent.spawn((
				DilemmaTimerPosition,
				DilemmaTimer::new(
					dilemma.countdown_duration, 
					Duration::from_secs_f32(5.0),
					Duration::from_secs_f32(2.0)
				
				),
				ColorAnchor::default(),
				ColorChangeOn::new(vec![
					ColorChangeEvent::Pulse(vec![DANGER_COLOR])
				]),
				Transform::from_xyz(0.0, -100.0, 1.0)
			));

			parent.spawn((
				Lever(state),
				ClickablePong::new(vec![
						vec![LeverActions::RightPull],
						vec![LeverActions::LeftPull]
					]					
				),
				Pressable::new(vec![
					KeyMapping{
						keys : vec![KeyCode::Digit2], 
						actions : vec![LeverActions::RightPull],
						allow_repeated_activation : false
					},
					KeyMapping{
						keys : vec![KeyCode::Digit1],
						actions : vec![LeverActions::LeftPull],
						allow_repeated_activation : false
					}
				]),
				ActionPallet(
					enum_map!(
						LeverActions::LeftPull => vec![
							InputAction::ChangeLeverState(LeverState::Left),
							InputAction::PlaySound(DilemmaSounds::Lever),
						],
						LeverActions::RightPull => vec![
							InputAction::ChangeLeverState(LeverState::Right),
							InputAction::PlaySound(DilemmaSounds::Lever),
						]
					)
				),
				CenterLever,
				Text2d::new(start_text), 
				TextFont{
					font_size : 25.0,
					..default()
				},
				TextColor(color),
				TextLayout{
					justify : JustifyText::Center, 
					..default()
				},
				TransientAudioPallet::new(
					vec![(
						DilemmaSounds::Lever,
						vec![
							TransientAudio::new(
								asset_server.load("sounds/switch.ogg"), 
								0.1, 
								true,
								1.0
							)
						],
						Some(DilatableAudio)
					)]
				),
			));
		});
}


#[derive(Component)]
pub struct DilemmaConsequenceRoot;

pub fn setup_dilemma_consequence_animation(
	mut commands : Commands,
	mut velocity_query : Query<&mut Velocity, With<Train>>,
	asset_server: Res<AssetServer>
){

	for mut velocity in velocity_query.iter_mut() {
		velocity.0 = Vec3::new(100.0, 0.0, 0.0);
	}
	
	commands.spawn((
		DilemmaConsequenceRoot,
		TimerPallet::new(
			vec![
				(
					"speedup".to_string(),
					TimerConfig::new(
						TimerStartCondition::Immediate, 
						3.0,
						None
					)
				),
				(
					"scream".to_string(),
					TimerConfig::new(
						TimerStartCondition::Immediate, 
						1.0,
						None
					)
				)
			]
		))
	).with_children( 
		|parent| 
		{
			parent.spawn(
				OneShotAudioPallet::new(
					vec![
						(OneShotAudio {
							source : asset_server.load(
								PathBuf::from("./sounds/slowmo.ogg")
							),
							persistent : false,
							volume :1.0
						}, None)
					]
				)
			);

			parent.spawn(
				DilationTranslation::new(
					0.1, 
					Duration::from_secs_f32(1.0)
				)
			);
		}
	);
}

fn spawn_delayed_children_consequence(
    mut commands: Commands,
    loading_query: Query<(Entity, &TimerPallet), With<DilemmaConsequenceRoot>>,
    asset_server: Res<AssetServer>
) {
    for (entity, timers) in loading_query.iter() {
		if let Some(timer) = timers.timers.get(
            "scream"
        ) {
            if timer.just_finished() {
				commands.entity(entity).with_children(
                    |parent| {
                    
					parent.spawn(
						OneShotAudioPallet::new(
							vec![
								(OneShotAudio {
									source : asset_server.load(
										PathBuf::from(
											"./sounds/male_scream_long.ogg"
										)
									),
									persistent : false,
									volume :1.0
								}, Some(DilatableAudio))
							]
						)
					);
                });

            }
        } else {
            warn!("Entity {:?} is missing the 'scream' timer", entity);
        }

		if let Some(timer) = timers.timers.get(
            "speedup"
        ) {
            if timer.just_finished() {
				commands.entity(entity).with_children(
                    |parent| {

					parent.spawn(
						DilationTranslation::new(
							1.0, 
							Duration::from_secs_f32(1.057)
						)
					);
                    
					parent.spawn(
						OneShotAudioPallet::new(
							vec![
								(OneShotAudio {
									source : asset_server.load(
										PathBuf::from("./sounds/speedup.ogg")
									),
									persistent : false,
									volume :1.0
								}, None)
							]
						)
					);
                });

            }
        } else {
            warn!("Entity {:?} is missing the 'speedup' timer", entity);
        }
    }
}
