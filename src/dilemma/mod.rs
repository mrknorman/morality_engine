use std::{
	path::PathBuf,
	time::Duration
};
use bevy::{
	prelude::*,
	audio::Volume,
	sprite::Anchor
};
use enum_map::Enum;
use phases::{
	consequence::DilemmaConsequencePlugin, 
	decision::DilemmaDecisionPlugin, 
	intro::DilemmaIntroPlugin, 
	transition::DilemmaTransitionPlugin
};
use crate::{
	ascii_fonts::{
		AsciiPlugin, 
		AsciiString
	}, audio::{
		continuous_audio, 
		MusicAudio
	}, background::{
		Background, 
		BackgroundPlugin
	}, colors::{
		ColorTranslation, 
		Fade,
		BACKGROUND_COLOR, 
		DIM_BACKGROUND_COLOR, 
		OPTION_1_COLOR, 
		OPTION_2_COLOR, 
		PRIMARY_COLOR
	}, 
	game_states::{
		DilemmaPhase, 
		GameState, 
	}, 
	inheritance::BequeathTextColor, 
	interaction::InteractionPlugin,
	io::IOPlugin, 
	motion::PointToPointTranslation,
	person::PersonPlugin, 
	physics::Velocity, 
	timing::TimingPlugin, 
	train::{
        Train, 
		TrainPlugin, 
		STEAM_TRAIN
    }
};

pub mod phases;

mod dilemma;
use dilemma::{
	Dilemma, 
	DilemmaPlugin
};
pub mod lever;
use lever::LeverPlugin;
mod junction;
use junction::{
	Junction, 
	JunctionPlugin
};

pub struct DilemmaScenePlugin;
impl Plugin for DilemmaScenePlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(GameState::Dilemma), 
			setup_dilemma
		)
		.add_systems(
			OnEnter(DilemmaPhase::Results), 
			setup_results
		);
		
		app.add_plugins(DilemmaIntroPlugin);
		app.add_plugins(DilemmaTransitionPlugin);
		app.add_plugins(DilemmaDecisionPlugin);
		app.add_plugins(DilemmaConsequencePlugin);

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
pub enum DilemmaConsequenceActions {
    ShowResults
}

impl std::fmt::Display for DilemmaConsequenceActions {
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
				Fade{
					duration : transition_duration, 
					paused : true
				},
				Transform::from_xyz(-400.0,300.0, 1.0)
			));

			parent.spawn((
				TextColor(PRIMARY_COLOR),
				Fade{
					duration : transition_duration,
					paused : true
				},
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
					0.00002,
					-0.5
				),
				BequeathTextColor,
				ColorTranslation::new( //something to do with this???
					DIM_BACKGROUND_COLOR,
					transition_duration,
					true
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
					transition_duration,
					true
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
					transition_duration,
					true
				),
				Transform::from_translation(main_track_translation_start),
				PointToPointTranslation::new(
					main_track_translation_end,
					transition_duration,
					true
				)
			));
		}
	);

	commands.insert_resource(dilemma);
}


#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaSounds {
	TrainApproaching,
	Clock,
	Click,
	Lever
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct ResultsRoot;

fn setup_results(
	mut commands: Commands,
	mut train_query : Query<(Entity, &mut Velocity), With<Train>>,
	asset_server: Res<AssetServer>
) {

	commands.spawn(
		ResultsRoot
	).with_children(|parent| {
		parent.spawn((
			MusicAudio,
			AudioPlayer::<AudioSource>(asset_server.load(
				"./music/the_right_track.ogg"
			)),
			PlaybackSettings{
				paused : false,
				volume : Volume::new(0.3),
				..continuous_audio()
			}
		));
		
		parent.spawn((
			TextColor(PRIMARY_COLOR),
			AsciiString(format!("DILEMMA RESULTS")),
			Transform::from_xyz(-550.0,300.0, 1.0)
		));
		}
	);

	

	for (entity, mut velocity) in train_query.iter_mut() {
		velocity.0 = Vec3::ZERO;
		commands.entity(entity).insert(
			PointToPointTranslation::new(
				Vec3::new(120.0, 150.0, 0.0),
				Duration::from_secs_f32(0.1),
				false
			)
		);
	}
}