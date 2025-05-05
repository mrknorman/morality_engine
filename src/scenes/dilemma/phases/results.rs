use std::time::Duration;

use bevy::{
    audio::Volume,
	prelude::*
};
use enum_map::{enum_map, Enum};

use crate::{
	data::{
		stats:: GameStats, 
		states::DilemmaPhase
	},
	systems::{
		audio::{
        	continuous_audio, 
			MusicAudio, 
			TransientAudio, 
			TransientAudioPallet
    	}, 
		colors::{
			ColorTranslation, 
			DIM_BACKGROUND_COLOR, 
			PRIMARY_COLOR
		},
		interaction::{
			ActionPallet, 
			Draggable, 
			InputAction
		},
		physics::Velocity,
		backgrounds::{
			Background,
			content::BackgroundTypes
		},
		inheritance::BequeathTextColor
	},	
	entities::{
		large_fonts::{AsciiString, TextEmotion},
		sprites::window::WindowTitle,
		text::{
			TextButton, 
			WindowedTable
		}, 
		train::Train
	},
	style::common_ui::NextButton, 
	scenes::dilemma::DilemmaSounds, 
};

pub struct DilemmaResultsPlugin;
impl Plugin for DilemmaResultsPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
            OnEnter(DilemmaPhase::Results), 
            DilemmaResultsScene::setup,
        );
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaResultsActions {
    ExitResults
}

impl std::fmt::Display for DilemmaResultsActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DilemmaResultsScene;

impl DilemmaResultsScene {

	const TEXT_BOX_Z : f32 = 1.0;
	
	fn setup(
		mut commands: Commands,
		mut train_query : Query<(&mut Transform, &mut Velocity), With<Train>>,
		stats : Res<GameStats>,
		asset_server: Res<AssetServer>,
	) {
	
		commands.spawn((
			Self,
			StateScoped(DilemmaPhase::Results),
			children![
				(
					Draggable::default(),
					WindowedTable{
						title : Some(WindowTitle{
							text : String::from("Overall Results"),
							..default()
						}),
						..default()
					},
					stats.to_table(),
					Transform::from_xyz(
						50.0,
						0.0,
						Self::TEXT_BOX_Z + 0.2,
					)
				),
				(
					TextColor(Color::NONE),
					Background::new(
						BackgroundTypes::Desert,	
						0.00002,
						-0.5
					),
					BequeathTextColor,
					ColorTranslation::new(
						DIM_BACKGROUND_COLOR,
						Duration::from_secs_f32(0.2),
						false
					)
				),
				(
					MusicAudio,
					AudioPlayer::<AudioSource>(asset_server.load(
						"./audio/music/the_right_track.ogg"
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
					AsciiString(format!("DILEMMA RESULTS")),
					Transform::from_xyz(-550.0,300.0, 1.0)
				),
				(
					NextButton,
					TextButton::new(
						vec![DilemmaResultsActions::ExitResults],
						vec![KeyCode::Enter],
						"[ Click here or Press Enter to End the Simulation ]",
					),
					ActionPallet::<DilemmaResultsActions, DilemmaSounds>(
						enum_map!(
							DilemmaResultsActions::ExitResults => vec![
								InputAction::PlaySound(DilemmaSounds::Click),
								InputAction::NextScene,
								InputAction::Despawn(None)
						])
					),
					TransientAudioPallet::new(
						vec![(
							DilemmaSounds::Click,
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
					)
				),
				(
					Draggable::default(),
					WindowedTable{
						title : Some(WindowTitle{
							text : String::from("Latest Results"),
							..default()
						}),
						..default()
					},
					stats.dilemma_stats.last().cloned().expect("No last dilemma").to_table(),
					Transform::from_xyz(
						-450.0,
						0.0,
						Self::TEXT_BOX_Z + 0.2,
					)
				)
			]
		));
	
		for (mut transform, mut velocity) in train_query.iter_mut() {
			velocity.0 = Vec3::ZERO;
            transform.translation = Vec3::new(120.0, 150.0, 0.0);
		}
	}
}



