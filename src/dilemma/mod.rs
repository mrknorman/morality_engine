
use std::path::PathBuf;

use bevy::prelude::*;

use crate::{
	lever::check_level_pull, 
	train::Train,
	track::Track,
	background::{
		BackgroundSprite,
		LARGE_CACTUS,
		SMALL_CACTUS
	},
	narration::{
		start_narration,
		Narration
	},
	audio::play_sound_once,
	motion::{
		PointToPointTranslation,
		Locomotion
	},
	game_states::{
		SubState, 
		MainState,
		GameState
	},
	io_elements::{
		check_if_enter_pressed, 
		spawn_text_button, 
		show_text_button, 
		text_button_interaction
	},
};

mod dilemma;
use dilemma::{
	TrainJunction,
	end_transition,
	person_check_danger,
	animate_person,
	lever_motion,
	update_timer,
	cleanup_decision,
	consequence_animation_tick_up,
	consequence_animation_tick_down,
	Dilemma,
	DramaticPauseTimer,
	DilemmaInfoPanel,
	DilemmaHeader,
	TransitionCounter,
	DilemmaDashboard
};
	
pub struct DilemmaPlugin;
impl Plugin for DilemmaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Dilemma), setup_dilemma_intro)
            .add_systems(
                Update,
                (
                    check_if_enter_pressed
                )
                    .run_if(in_state(GameState::Dilemma))
                    .run_if(in_state(SubState::Intro)),
            )
            .add_systems(
                Update,
                (
                    start_narration,
                    show_text_button,
                    text_button_interaction,
                    BackgroundSprite::move_background_spites,
                )
                .run_if(in_state(GameState::Dilemma)),
            )
            .add_systems(OnEnter(SubState::IntroDecisionTransition), setup_dilemma_transition)
            .add_systems(
                Update,
                (end_transition)
                    .run_if(in_state(GameState::Dilemma))
                    .run_if(in_state(SubState::IntroDecisionTransition)),
            )
            .add_systems(
                OnEnter(SubState::Decision),
                setup_decision.run_if(in_state(GameState::Dilemma)),
            )
            .add_systems(
                Update,
                (
                    check_level_pull,
                    person_check_danger,
                    animate_person,
                    lever_motion,
                    update_timer
                )
                    .run_if(in_state(GameState::Dilemma))
                    .run_if(in_state(SubState::Decision)),
            )
            .add_systems(OnExit(SubState::Decision), cleanup_decision)
            .add_systems(
                OnEnter(SubState::ConsequenceAnimation),
                setup_dilemma_consequence_animation,
            )
            .add_systems(
                Update,
                (
                    consequence_animation_tick_up,
                    consequence_animation_tick_down,
                )
                    .run_if(in_state(GameState::Dilemma))
                    .run_if(in_state(SubState::ConsequenceAnimation)),
            );//.add_plugins(TrainPlugin::new(GameState::Dilemma));
    }
}

pub fn setup_dilemma_intro(
	mut commands : Commands,
	asset_server: Res<AssetServer>
	) {

	commands.spawn(
		(
			StateScoped(GameState::Dilemma),
			TransformBundle::from_transform(Transform::from_translation(
				Vec3::new(0.0, 0.0, 0.0))
			),
			VisibilityBundle::default()
		)
	);

	BackgroundSprite::spawn_multi(
		&mut commands, 
		SMALL_CACTUS, 
		LARGE_CACTUS,
		".",
		0.5,
		5
	);

	let dilemma : Dilemma = Dilemma::load(
		PathBuf::from("./dilemmas/lab_1.json")
	);
	let dilemma_entity : Entity = DilemmaInfoPanel::spawn(
		&mut commands, &dilemma
	);

	let narration_audio_entity : Entity = commands.spawn((
		Narration {
			timer: Timer::from_seconds(1.0, TimerMode::Once)
		},
		AudioBundle {
			source: asset_server.load(
				PathBuf::from("sounds/dilemma_narration/lab_1.ogg")
			),
			settings : PlaybackSettings {
				paused : true,
				volume : bevy::audio::Volume::new(1.0),
				mode:  bevy::audio::PlaybackMode::Remove,
				..default()
			}
		})).id();

	TrainJunction::spawn(&mut commands, &asset_server, &dilemma);

	let music_audio: Entity = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./music/algorithm_of_fate.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.3),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

	let button_entity = spawn_text_button(
		"[Click here or Press Enter to Begin]",
		Some(MainState::InGame),
		Some(GameState::Dilemma),
		Some(SubState::IntroDecisionTransition),
		2.0,
		&mut commands
	);

	commands.insert_resource(dilemma);
	commands.insert_resource(DilemmaHeader{
		button_entity,
		dilemma_entity,
		narration_audio_entity
	});

}

pub fn setup_dilemma_transition(
	mut commands : Commands,
	background_query : Query<&mut BackgroundSprite>,
	dilemma: Res<Dilemma>,  // Add time resource to manage frame delta time
	entities : ResMut<DilemmaHeader>,
	locomotion_query : Query<&mut Locomotion, With<Train>>,
	mut track_query: Query<&mut PointToPointTranslation, With<Track>>
) {

	commands.entity(entities.button_entity).despawn_recursive();
	commands.entity(entities.narration_audio_entity).despawn_recursive();

	let speed: f32 = -450.0;
	let decision_position = -45.0 * dilemma.countdown_duration_seconds;
	let duration_seconds = decision_position/speed;

	BackgroundSprite::update_speed(background_query,2.0);
	Train::update_speed(
		locomotion_query,
		speed
	);

	let transition_timer = TransitionCounter{
		timer : Timer::from_seconds(duration_seconds, TimerMode::Once)
	};

	for mut track in track_query.iter_mut() {
		track.set_duration(duration_seconds);
		track.start()
	}

	commands.insert_resource(transition_timer);
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

pub fn setup_decision(
		mut commands : Commands,
		asset_server: Res<AssetServer>,
		dilemma: Res<Dilemma>,  // Add time resource to manage frame delta time
	) {
	
	let train_audio = commands.spawn(AudioBundle {
		source: asset_server.load(
			PathBuf::from("./sounds/train_aproaching.ogg")
		),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(1.0),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

	let clock_audio: Entity = commands.spawn(AudioBundle {
		source: asset_server.load(PathBuf::from("./sounds/clock.ogg")),
		settings : PlaybackSettings {
			paused : false,
			volume : bevy::audio::Volume::new(0.3),
			mode:  bevy::audio::PlaybackMode::Loop,
			..default()
	}}).id();

	DilemmaDashboard::spawn(&mut commands, &dilemma);
}