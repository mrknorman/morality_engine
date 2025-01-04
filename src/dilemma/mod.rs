use std::{
	path::PathBuf,
	time::Duration
};
use bevy::{
	prelude::*,
	audio::Volume,
	sprite::Anchor
};
use crate::{
	ascii_fonts::AsciiString, 
	audio::{
		continuous_audio, 
		one_shot_audio, 
		play_sound_once,
		ContinuousAudioPallet,
		MusicAudio, 
		NarrationAudio,
		TransientAudio,
		TransientAudioPallet
	}, 
	background::{
		Background, 
		BackgroundPlugin, 
		BackgroundSystems
	}, 
	colors::{
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
	}, 
	common_ui::{
		CenterLever, DilemmaTimerPosition, NextButton
	}, 
	game_states::{
		DilemmaPhase, 
		GameState, 
		MainState, 
		StateVector
	},
	inheritance::BequeathTextColor, 
	interaction::{
		InputAction, 
		InteractionPlugin
	}, 
	motion::{
		Bounce, 
		PointToPointTranslation, Pulse
	}, 
	person::PersonPlugin, 
	text::{TextButton, TextRaw}, 
	timing::{
        TimerConfig, 
		TimerPallet, 
		TimerStartCondition,
		TimingPlugin
    }, train::{
        Train,
        STEAM_TRAIN
    }
};

mod dilemma;
use dilemma::{
	cleanup_decision,
	consequence_animation_tick_up,
	consequence_animation_tick_down,
	Dilemma,
	DramaticPauseTimer,
	DilemmaTimer,
};
mod lever;
use lever::{
	Lever, LeverPlugin, LeverState, LEVER_LEFT, LEVER_MIDDLE, LEVER_RIGHT
};
mod junction;
use junction::{
	Junction, JunctionPlugin, TrunkTrack
};

pub struct DilemmaPlugin;
impl Plugin for DilemmaPlugin {
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
			Update,
			(
				DilemmaTimer::update,
				DilemmaTimer::start_pulse
			)
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::Decision)),
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
			(
				consequence_animation_tick_up,
				consequence_animation_tick_down,
			)
			.run_if(in_state(GameState::Dilemma))
			.run_if(in_state(DilemmaPhase::ConsequenceAnimation)),
		)
		.register_required_components::<Junction, Transform>()
		.register_required_components::<Junction, Visibility>()
		.register_required_components::<DilemmaTimer, TextRaw>()
		.register_required_components::<DilemmaTimer, Text2d>()
		.register_required_components::<DilemmaTimer, BequeathTextColor>()
		.register_required_components::<DilemmaTimer, Pulse>()
		;

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
				ColorTranslation::new(
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
                let next_state_vector = StateVector::new(
                    Some(MainState::InGame),
                    Some(GameState::Dilemma),
                    Some(DilemmaPhase::IntroDecisionTransition),
                );
                
				commands.entity(entity).with_children(|parent| {
					parent.spawn((
						NextButton,
						TextButton::new(
							vec![
								InputAction::PlaySound(String::from("click")),
								InputAction::ChangeState(next_state_vector),
								InputAction::Despawn
							],
							vec![KeyCode::Enter],
							"[ Click here or Press Enter to Test Your Morality ]",
						),
						TransientAudioPallet::new(
							vec![(
								"click".to_string(),
								vec![
									TransientAudio::new(
										asset_server.load("sounds/mech_click.ogg"), 
										0.1, 
										true,
										1.0
									)
								]
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
		mut background_query : Query<(&mut Background, &mut ColorTranslation), Without<TrunkTrack>>,
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
			translation.final_position = initial_position - Vec3::new(60.0, 0.0, 0.0);
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

	commands.insert_resource(Lever(state.clone()));

	commands.spawn((
		StateScoped(DilemmaPhase::Decision),
		DecisionRoot
	)).with_children(
        |parent| {
            parent.spawn(
                ContinuousAudioPallet::new(
                    vec![
                        (
                            "train_aproaching".to_string(),
                            AudioPlayer::<AudioSource>(asset_server.load(
                                "./sounds/train_aproaching.ogg"
                            )),
                            PlaybackSettings{
                                volume : Volume::new(1.0),
                                ..continuous_audio()
                            }
                        ),
                        (
                            "office".to_string(),
                            AudioPlayer::<AudioSource>(asset_server.load(
                                "./sounds/clock.ogg"
                            )),
                            PlaybackSettings{
                                volume : Volume::new(0.3),
                                ..continuous_audio()
                            }
                        )
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
				ColorChangeOn::new(vec![ColorChangeEvent::Pulse(vec![DANGER_COLOR])]),
				Transform::from_xyz(0.0, -100.0, 1.0)
			));

			parent.spawn((
				Lever(state.clone()),
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
						"lever".to_string(),
						vec![
							TransientAudio::new(
								asset_server.load("sounds/switch.ogg"), 
								0.1, 
								true,
								1.0
							)
						]
					)]
				),
			));
		});
}

pub fn setup_dilemma_consequence_animation(
	mut commands : Commands,
	asset_server: Res<AssetServer>
){
	play_sound_once("./sounds/slowmo.ogg", &mut commands, &asset_server);

	commands.insert_resource(DramaticPauseTimer{
		speed_up_timer: Timer::from_seconds(4.0, TimerMode::Once),
		scream_timer: Timer::from_seconds(3.0, TimerMode::Once)
	});
}
