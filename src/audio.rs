use bevy::prelude::*;
use std::{path::PathBuf};

pub fn play_sound_once(
	audio_path : &str,
	commands: &mut Commands, 
	asset_server : &Res<AssetServer>
) {

	commands.spawn((
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