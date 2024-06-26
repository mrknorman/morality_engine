use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Component)]

struct SingleSound;

pub fn play_sound_once(
	audio_path : &str,
	commands: &mut Commands, 
	asset_server : &Res<AssetServer>
) -> Entity {

	commands.spawn(
		(SingleSound,
		AudioBundle {
			source: asset_server.load(PathBuf::from(audio_path)),
			settings : PlaybackSettings {
				paused : false,
				mode:  bevy::audio::PlaybackMode::Despawn,
				volume :bevy::audio::Volume::new(0.5),
				..default()
			}
		}
	)).id()
}

#[derive(Resource)]
pub struct BackgroundAudio {
	pub audio : Vec<Entity>
}
