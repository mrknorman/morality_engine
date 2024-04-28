use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Component)]

struct SingleSound;

pub fn play_sound_once(
	audio_path : &str,
	commands: &mut Commands, 
	asset_server : &Res<AssetServer>
) {

	commands.spawn(
		(SingleSound,
		AudioBundle {
			source: asset_server.load(PathBuf::from(audio_path)),
			settings : PlaybackSettings {
				paused : false,
				mode:  bevy::audio::PlaybackMode::Despawn,
				..default()
			}
		}
	));
}

#[derive(Resource)]
pub struct BackgroundAudio {
	pub audio : Vec<Entity>
}
