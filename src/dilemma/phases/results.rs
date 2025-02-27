use std::time::Duration;

use bevy::{
    audio::Volume,
	prelude::*
};

use crate::{
    ascii_fonts::AsciiString, audio::{
        continuous_audio, 
        MusicAudio
    }, background::Background, colors::{
		ColorTranslation, 
		DIM_BACKGROUND_COLOR, 
		PRIMARY_COLOR
	}, game_states::DilemmaPhase, inheritance::BequeathTextColor, motion::PointToPointTranslation, physics::Velocity, stats::DilemmaStats, text::TextBox, train::Train
};


pub struct DilemmaResultsPlugin;
impl Plugin for DilemmaResultsPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
            OnEnter(DilemmaPhase::Results), 
            DilemmaResultsScene::setup
        );
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DilemmaResultsScene;

impl DilemmaResultsScene {
	fn setup(
		mut commands: Commands,
		mut train_query : Query<(&mut Transform, &mut Velocity), With<Train>>,
		stats : Res<DilemmaStats>,
		asset_server: Res<AssetServer>,
	) {
	
		commands.spawn(
			Self
		).with_children(|parent| {

			let text_box_z : f32 = 1.0; 

			parent.spawn((
				TextBox::default(),
                TextColor(Color::NONE),
				TextFont{
					font_size : 15.0,
					..default()
				},
                Text2d::new(stats.to_string()),
                ColorTranslation::new(
                    PRIMARY_COLOR,
                    Duration::from_secs_f32(0.2),
                    false
                ),
				Transform::from_xyz(
					0.0,
					0.0,
					text_box_z + 0.2,
				))
            );

            parent.spawn((
                TextColor(Color::NONE),
                Background::load_from_json(
                    "text/backgrounds/desert.json",	
                    0.00002,
                    -0.5
                ),
                BequeathTextColor,
                ColorTranslation::new(
                    DIM_BACKGROUND_COLOR,
                    Duration::from_secs_f32(0.2),
                    false
                ))
            );

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
	
		for (mut transform, mut velocity) in train_query.iter_mut() {
			velocity.0 = Vec3::ZERO;
            transform.translation = Vec3::new(120.0, 150.0, 0.0);
		}
	}
}



