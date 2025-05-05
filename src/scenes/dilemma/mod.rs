use std::time::Duration;
use bevy::{
	audio::Volume, prelude::*, sprite::Anchor, text::TextBounds
};
use enum_map::Enum;
use phases::{
	consequence::DilemmaConsequencePlugin, 
	decision::DilemmaDecisionPlugin,
	intro::DilemmaIntroPlugin, 
	results::DilemmaResultsPlugin, 
	transition::DilemmaTransitionPlugin
};
use crate::{
	data::{
		states::GameState, stats::DilemmaStats 
	}, entities::{
		large_fonts::{
			AsciiPlugin, 
			AsciiString, 
			TextEmotion
		}, person::PersonPlugin, sprites::{
			window::WindowTitle, 
			SpritePlugin
		}, text::{
			TextPlugin, 
			TextWindow
		}, train::{
			content::TrainTypes, Train, TrainPlugin
		} 
	}, style::ui::IOPlugin, systems::{
		audio::{
			continuous_audio, 
			MusicAudio
		}, backgrounds::{
			content::BackgroundTypes, Background, BackgroundPlugin
		}, colors::{
			AlphaTranslation, ColorTranslation, Fade, BACKGROUND_COLOR, DIM_BACKGROUND_COLOR, OPTION_1_COLOR, OPTION_2_COLOR, PRIMARY_COLOR
		}, inheritance::BequeathTextAlpha, interaction::{
			Draggable, 
			InteractionPlugin
		}, motion::PointToPointTranslation, scheduling::TimingPlugin
	} 
};

pub mod phases;

pub mod dilemma;
use dilemma::{
	Dilemma, 
	DilemmaPlugin
};
pub mod lever;
pub mod content;
use content::DilemmaScene;
use lever::LeverPlugin;
mod junction;
use junction::{
	Junction, 
	JunctionPlugin
};

use super::{SceneQueue, Scene};

pub struct DilemmaScenePlugin;
impl Plugin for DilemmaScenePlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			OnEnter(GameState::Dilemma), 
			DilemmaScene::setup
		)
		.add_plugins(DilemmaIntroPlugin)
		.add_plugins(DilemmaTransitionPlugin)
		.add_plugins(DilemmaDecisionPlugin)
		.add_plugins(DilemmaConsequencePlugin)
		.add_plugins(DilemmaResultsPlugin);

		if !app.is_plugin_added::<SpritePlugin>() {
			app.add_plugins(SpritePlugin);
		}
		if !app.is_plugin_added::<TextPlugin>() {
			app.add_plugins(TextPlugin);
		}
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

impl DilemmaScene {

	const TRAIN_SPEED : f32 = -1000.0;
	const TRAIN_INITIAL_POSITION : Vec3 = Vec3::new(120.0, -10.0, 1.0);
	const MAIN_TRACK_TRANSLATION_END : Vec3 = Vec3::new(0.0, -40.0, 0.0);
	const TRACK_COLORS : [Color; 2] = [OPTION_1_COLOR, OPTION_2_COLOR];

	fn setup(
		mut commands : Commands,
		queue : Res<SceneQueue>,
		asset_server: Res<AssetServer>
	) {

		let scene = queue.current;

		let dilemma = match scene {
			Scene::Dilemma(content) => {
				Dilemma::new(&content)
			},
			_ => panic!("Scene is not dilemma!") 
		};

		commands.insert_resource(
			DilemmaStats::new(dilemma.countdown_duration)
		);

		let decision_position = -70.0 * dilemma.countdown_duration.as_secs_f32();
		let transition_duration = Duration::from_secs_f32(decision_position/Self::TRAIN_SPEED);
		let train_x_displacement = Vec3::new(decision_position, 0.0, 0.0);
		let final_position = Vec3::new(
			150.0 * dilemma.countdown_duration.as_secs_f32(),
			0.0, 
			0.0
		);
		let main_track_translation_start: Vec3 = Self::MAIN_TRACK_TRANSLATION_END + final_position;
		let initial_color = match dilemma.default_option {
			None => Color::WHITE,
			Some(ref option) => Self::TRACK_COLORS[*option]
		};
		
		commands.spawn(
			(
				scene,
				StateScoped(GameState::Dilemma),
				children![
					(
						MusicAudio,
						AudioPlayer::<AudioSource>(asset_server.load(
							"./audio/music/algorithm_of_fate.ogg"
						)),
						PlaybackSettings{
							paused : false,
							volume : Volume::Linear(0.3),
							..continuous_audio()
						}
					),
					(
						TextColor(PRIMARY_COLOR),
						TextEmotion::Happy,
						AsciiString(format!("DILEMMA {}", dilemma.index)),
						Fade{
							duration : transition_duration, 
							paused : true
						},
						Transform::from_xyz(-400.0,300.0, 1.0)
					),
					(
						TextWindow{
							title : Some(WindowTitle{
								text : format!("Description: {}" , dilemma.name.clone()),
								..default()
							}),
							..default()
						},
						TextBounds {
							width : Some(400.0), 
							height : None
						},
						Draggable::default(),
						TextColor(PRIMARY_COLOR),
						Text2d::new(&dilemma.description),
						TextFont{
							font_size : 12.0,
							..default()
						},
						Anchor::TopLeft,
						Transform::from_xyz(-600.0,200.0, 2.0)
					),
					(
						TextColor(BACKGROUND_COLOR),
						Background::new(
							BackgroundTypes::Desert,	
							0.00002,
							-0.5
						),
						BequeathTextAlpha,
						AlphaTranslation::new(
							DIM_BACKGROUND_COLOR.alpha(),
							transition_duration,
							true
						)
					),
					(
						Train(TrainTypes::SteamTrain),
						Transform::from_translation(Self::TRAIN_INITIAL_POSITION), 
						PointToPointTranslation::new(
							Self::TRAIN_INITIAL_POSITION + train_x_displacement,
							transition_duration,
							true
						)
					),
					(
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
							Self::MAIN_TRACK_TRANSLATION_END,
							transition_duration,
							true
						)
					)	
				]
			)
		);

		commands.insert_resource(dilemma);
	}	
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaSounds {
	TrainApproaching,
	Clock,
	Click,
	Lever
}
