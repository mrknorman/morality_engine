use std::time::Duration;
use bevy::{
	audio::Volume, prelude::*, sprite::Anchor, text::TextBounds
};
use enum_map::Enum;
use phases::{
	consequence::DilemmaConsequencePlugin, decision::DilemmaDecisionPlugin, intro::DilemmaIntroPlugin, results::DilemmaResultsPlugin, skip::DilemmaSkipPlugin
};
use crate::{
	data::{
		states::{DilemmaPhase, GameState}, stats::DilemmaStats 
	}, entities::{
		large_fonts::{
			AsciiPlugin, 
			AsciiString, 
			TextEmotion
		}, person::PersonPlugin, sprites::{
			SpritePlugin, window::WindowTitle
		}, text::{
			TextPlugin, 
			TextWindow
		}, track::Track, train::{
			Train, TrainPlugin, content::TrainTypes
		} 
	}, scenes::dilemma::{dilemma::{CurrentDilemmaStageIndex, DilemmaStage}, phases::transition::DilemmaTransitionPlugin}, style::ui::IOPlugin, systems::{
		audio::{
			MusicAudio, continuous_audio
		}, backgrounds::{
			Background, BackgroundPlugin, content::BackgroundTypes
		}, colors::{
			AlphaTranslation, BACKGROUND_COLOR, DIM_BACKGROUND_COLOR, Fade, OPTION_1_COLOR, OPTION_2_COLOR, PRIMARY_COLOR
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
		.add_plugins(DilemmaDecisionPlugin)
		.add_plugins(DilemmaTransitionPlugin)
		.add_plugins(DilemmaConsequencePlugin)
		.add_plugins(DilemmaResultsPlugin)
		.add_plugins(DilemmaSkipPlugin);

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

		let total_dilemma_time: Duration = dilemma.stages
			.iter()
			.map(|s| s.countdown_duration)
			.sum();

		commands.insert_resource(
			DilemmaStats::new(total_dilemma_time)
		);

		commands.insert_resource(
			CurrentDilemmaStageIndex(0)
		);

		let stage: &dilemma::DilemmaStage = dilemma.stages.first().expect("Dilemma has no stages!");

		let (transition_duration, train_x_displacement, _, _) = Self::generate_common_parameters(stage);
		
		commands.spawn(
			(
				scene,
				StateScoped(GameState::Dilemma),
				children![
					(
						MusicAudio,
						AudioPlayer::<AudioSource>(asset_server.load(
							dilemma.music_path.clone()
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
							-0.5 * (dilemma.stages.first().expect("Dilemma has no stages").speed / 70.0)
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
						PointToPointTranslation::new(
							Self::TRAIN_INITIAL_POSITION,
							Self::TRAIN_INITIAL_POSITION + train_x_displacement,
							transition_duration,
							true
						)
					)
				]
			)
		);

		commands.spawn(
	(	
				StateScoped(DilemmaPhase::Intro),
				TextColor(BACKGROUND_COLOR),
				Track::new(2000),
				Transform::from_translation(Self::MAIN_TRACK_TRANSLATION_END)
			)
		);
		
		commands.insert_resource(dilemma.stages.first().expect("Dilemma has no stages!").clone());
		commands.insert_resource(dilemma);
	}	

	fn generate_common_parameters(stage : &DilemmaStage) -> (Duration, Vec3, Vec3, Color) {
		let decision_position = -stage.speed * stage.countdown_duration.as_secs_f32();
		let transition_duration = Duration::from_secs_f32(stage.countdown_duration.as_secs_f32() / 15.0); 
		let train_x_displacement = Vec3::new(decision_position, 0.0, 0.0);
		let final_position = Vec3::new(
			150.0 * stage.countdown_duration.as_secs_f32(),
			0.0, 
			0.0
		);
		let main_track_translation_start: Vec3 = Self::MAIN_TRACK_TRANSLATION_END + final_position;
		let initial_color = match stage.default_option {
			None => Color::WHITE,
			Some(ref option) => Self::TRACK_COLORS[*option]
		};

		(transition_duration, train_x_displacement, main_track_translation_start, initial_color)
	}
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaSounds {
	TrainApproaching,
	Clock,
	Click,
	Lever
}
