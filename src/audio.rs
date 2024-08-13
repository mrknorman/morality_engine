use bevy::{
    asset::AssetPath, audio::{
        PlaybackMode,
        Volume
    }, ecs::system::EntityCommands, prelude::*
};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Component)]
struct SingleSound;

pub fn play_sound_once(
    audio_path : &str,
    commands: &mut Commands, 
    asset_server : &Res<AssetServer>
) -> Entity {
    commands.spawn(
        (
            SingleSound,
            AudioBundle {
                source: asset_server.load(PathBuf::from(audio_path)),
                settings: PlaybackSettings {
                    paused: false,
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(0.5),
                    ..default()
                }
            }
        )
    ).id()
}

#[derive(Resource)]
pub struct BackgroundAudio {
    pub audio: HashMap<String, Entity>
}

#[derive(Component, Clone)]
pub struct ContinuousAudio;

impl ContinuousAudio {
    pub fn new<P: Into<AssetPath<'static>>>(
        audio_path: P,
        asset_server: &Res<AssetServer>,
        volume: f32
    ) -> impl Bundle {
        (
            ContinuousAudio,
            AudioBundle {
                source: asset_server.load(audio_path),
                settings: PlaybackSettings {
                    paused: false,
                    mode: PlaybackMode::Loop,
                    volume: Volume::new(volume),
                    ..default()
                }
            }
        )
    }
}

#[derive(Component)]
pub struct TransientAudio {
    source: Handle<AudioSource>,
    volume: f32
}

impl TransientAudio {
    pub fn new<P: Into<AssetPath<'static>>>(
        audio_path: P,
        asset_server: &Res<AssetServer>,
        volume: f32,
    ) -> TransientAudio {
        TransientAudio {
            source: asset_server.load(audio_path),
            volume,
        }
    }

    pub fn bundle(
        &self,
    ) -> impl Bundle {

		AudioBundle {
			source: self.source.clone(),
			settings: PlaybackSettings {
				paused: false,
				mode: PlaybackMode::Despawn,
				volume: Volume::new(self.volume),
				..default()
			}
		}
    }
}

#[derive(Component, Clone)]
pub struct AudioPallet;

#[derive(Component, Clone)]
pub struct ContinuousAudioPallet {
    pub entities: HashMap<String, Entity>
}

impl ContinuousAudioPallet {
    pub fn spawn(
        components: Vec<(String, impl Bundle)>,
        parent: &mut ChildBuilder<'_>
    ) -> impl Bundle {
        let mut entities: HashMap<String, Entity> = HashMap::new(); 
        for (name, audio_component) in components {
            let entity = parent.spawn(audio_component).id();
            entities.insert(name, entity);
        }

        (
            AudioPallet,
            ContinuousAudioPallet {
                entities
            }
        )
    }
}

#[derive(Component, Clone)]
pub struct TransientAudioPallet {
    pub entities: HashMap<String, Entity>
}

impl TransientAudioPallet {
    pub fn spawn(
        components: Vec<(String, impl Bundle)>,
        parent_entity: &mut EntityCommands
    ) -> impl Bundle {

		let mut entities: HashMap<String, Entity> = HashMap::new(); 

		parent_entity.with_children(
			|parent : &mut ChildBuilder<'_> | {
				for (name, audio_component) in components {
					let entity = parent.spawn(audio_component).id();
					entities.insert(name, entity);
				}
			}
		);
		
        (
            AudioPallet,
            TransientAudioPallet {
                entities
            }
        )
    }
	
	pub fn play_transient_audio(
        commands: &mut Commands, 
        entity: Entity,
        transient_audio : &TransientAudio
    ) {
		commands.entity(entity).with_children(
			| parent : &mut ChildBuilder<'_> | {
				parent.spawn(
					transient_audio.bundle()
				);
			}
		);
	}
}