
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
	person::PersonPlugin,
	audio::{
		play_sound_once, 
		continuous_audio, 
		one_shot_audio,
		MusicAudio, 
		NarrationAudio,
		TransientAudioPallet,
		TransientAudio
	}, 
	background::{Background, BackgroundPlugin, BackgroundSprite}, 
	common_ui::NextButton, 
	game_states::{
		DilemmaPhase, GameState, MainState, StateVector
	}, interaction::{
		InputAction, InteractionPlugin
	}, lever::check_level_pull, motion::{
		Locomotion, PointToPointTranslation
	}, text::TextButton, timing::{
        TimerConfig, TimerPallet, TimerStartCondition, TimingPlugin
    },
	train::{
        Train,
        STEAM_TRAIN
    }
};

mod dilemma;
use dilemma::{
	Junction,
	end_transition,
	check_if_person_in_path_of_train,
	switch_junction,
	update_timer,
	cleanup_decision,
	consequence_animation_tick_up,
	consequence_animation_tick_down,
	Dilemma,
	DramaticPauseTimer,
	DilemmaInfoPanel,
	TransitionCounter,
	DilemmaDashboard
};
pub struct DilemmaPlugin;
impl Plugin for DilemmaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dilemma), setup_dilemma)
			.add_systems(OnEnter(DilemmaPhase::Intro), setup_dilemma_intro
				.run_if(in_state(GameState::Dilemma))
			)
            .add_systems(
                Update,
                (
					spawn_delayed_children
                )
                    .run_if(in_state(GameState::Dilemma))
                    .run_if(in_state(DilemmaPhase::Intro)),
            ).add_systems(OnEnter(DilemmaPhase::IntroDecisionTransition), setup_dilemma_transition)
            .add_systems(
                Update,
                (end_transition)
                    .run_if(in_state(GameState::Dilemma))
                    .run_if(in_state(DilemmaPhase::IntroDecisionTransition)),
            ).add_systems(
                OnEnter(DilemmaPhase::Decision),
                setup_decision.run_if(in_state(GameState::Dilemma)),
            )
            .add_systems(
                Update,
                (
                    check_level_pull,
                    check_if_person_in_path_of_train,
                    switch_junction,
                    update_timer
                )
                    .run_if(in_state(GameState::Dilemma))
                    .run_if(in_state(DilemmaPhase::Decision)),
            )
			
			.add_systems(OnExit(DilemmaPhase::Decision), cleanup_decision)
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
            );

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

		app.register_required_components::<Junction, Transform>();
        app.register_required_components::<Junction, Visibility>();
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
			Background::load_from_json(
				"text/backgrounds/desert.json",	
				20.0,
				0.5
			),
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
			parent.spawn((
				DilemmaInfoPanel,
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
				Transform::from_xyz(0.0,300.0, 1.0)
			));	

			let speed: f32 = -450.0;
			let decision_position = -50.0 * dilemma.countdown_duration.as_secs_f32();
			let transition_duration = Duration::from_secs_f32(decision_position/speed);
			let train_initial_position = Vec3::new(120.0, -10.0, 1.0);
			let train_x_displacement = Vec3::new(-50.0 * dilemma.countdown_duration.as_secs_f32(), 0.0, 0.0);

			parent.spawn((
				Train::init(
					&asset_server,
					STEAM_TRAIN,
					train_initial_position,
					0.0
				),
				PointToPointTranslation::new(
					train_initial_position, 
					train_initial_position + train_x_displacement,
					transition_duration
				)
			));

			let final_position = Vec3::new(
				100.0 * dilemma.countdown_duration.as_secs_f32(),
				0.0, 
				0.0
			);

			let main_track_translation_end: Vec3 = Vec3::new(0.0, -40.0, 0.0);
			let main_track_translation_start: Vec3 = main_track_translation_end + final_position;

			parent.spawn((
				Junction{
					dilemma : dilemma.clone()
				},
				PointToPointTranslation::new(
					main_track_translation_start, 
					main_track_translation_end,
					transition_duration
				),
				Transform::from_translation(main_track_translation_start),
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

pub fn setup_dilemma_transition(
	mut commands : Commands,
	background_query : Query<&mut BackgroundSprite>,
	dilemma: Res<Dilemma>,  // Add time resource to manage frame delta time
	mut train_query : Query<&mut PointToPointTranslation, (With<Train>, Without<Junction>)>,
	mut junction_query: Query<&mut PointToPointTranslation, (With<Junction>, Without<Train>)>
) {

	let speed: f32 = -450.0;
	let decision_position = -50.0 * dilemma.countdown_duration.as_secs_f32();
	let duration_seconds = decision_position/speed;
	BackgroundSprite::update_speed(background_query,2.0);

	let transition_timer: TransitionCounter = TransitionCounter{
		timer : Timer::from_seconds(duration_seconds, TimerMode::Once)
	};

	for mut train in train_query.iter_mut() {
		train.start()
	}

	for mut track in junction_query.iter_mut() {
		track.start()
	}

	commands.insert_resource(transition_timer);
}

fn spawn_delayed_children(
    mut commands: Commands,
    loading_query: Query<(Entity, &TimerPallet), With<DilemmaIntroRoot>>,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>
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
						),
						NextButton::transform(&windows)
					)); // Capture the entity ID of the spawned child
				});
            }
        } else {
            warn!("Entity {:?} is missing the 'button' timer", entity);
        }
    }
}



pub fn setup_decision(
		mut commands : Commands,
		asset_server: Res<AssetServer>,
		dilemma: Res<Dilemma>,  // Add time resource to manage frame delta time
	) {
	
	commands.spawn((
		AudioPlayer::<AudioSource>(
			asset_server.load(
				PathBuf::from("./sounds/train_aproaching.ogg")
			)
		),
		PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(1.0),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
		})
	);

	commands.spawn((
		AudioPlayer::<AudioSource>(
			asset_server.load(PathBuf::from("./sounds/clock.ogg"))
		),
		PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.3),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
		})
	);
	
	DilemmaDashboard::spawn(&mut commands, &dilemma);
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
