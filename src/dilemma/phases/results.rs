use std::time::Duration;

use bevy::{audio::Volume, prelude::*};

use crate::{ascii_fonts::AsciiString, audio::{continuous_audio, MusicAudio}, colors::PRIMARY_COLOR, game_states::DilemmaPhase, motion::PointToPointTranslation, physics::Velocity, train::Train};


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
		mut train_query : Query<(Entity, &mut Velocity), With<Train>>,
		asset_server: Res<AssetServer>
	) {
	
		commands.spawn(
			Self
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
}