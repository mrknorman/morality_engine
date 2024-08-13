use bevy::{
    asset::AssetPath,
    audio::{PlaybackMode, Volume},
    ecs::system::EntityCommands,
    prelude::*,
};
use std::collections::HashMap;

#[derive(Component)]
struct SingleSound;

pub fn play_sound_once(
    audio_path: impl Into<AssetPath<'static>>,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) -> Entity {
    commands
        .spawn((
            SingleSound,
            AudioBundle {
                source: asset_server.load(audio_path),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(0.5),
                    ..default()
                },
            },
        ))
        .id()
}

#[derive(Resource)]
pub struct BackgroundAudio {
    pub audio: HashMap<String, Entity>,
}

#[derive(Component, Clone)]
pub struct ContinuousAudio;

impl ContinuousAudio {
    pub fn new(
        audio_path: impl Into<AssetPath<'static>>,
        asset_server: &Res<AssetServer>,
        volume: f32,
    ) -> impl Bundle {
        (
            ContinuousAudio,
            AudioBundle {
                source: asset_server.load(audio_path),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    volume: Volume::new(volume),
                    ..default()
                },
            },
        )
    }
}

#[derive(Component)]
pub struct TransientAudio {
    source: Handle<AudioSource>,
    volume: f32,
}

impl TransientAudio {
    pub fn new(
        audio_path: impl Into<AssetPath<'static>>,
        asset_server: &Res<AssetServer>,
        volume: f32,
    ) -> Self {
        Self {
            source: asset_server.load(audio_path),
            volume,
        }
    }

    pub fn bundle(&self) -> AudioBundle {
        AudioBundle {
            source: self.source.clone(),
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(self.volume),
                ..default()
            },
        }
    }
}

#[derive(Component, Clone)]
pub struct AudioPallet;

#[derive(Component, Clone)]
pub struct ContinuousAudioPallet {
    pub entities: HashMap<String, Entity>,
}

impl ContinuousAudioPallet {
    pub fn insert(components: Vec<(String, impl Bundle)>, parent_entity: &mut EntityCommands) {
        let entities = Self::spawn_children(components, parent_entity);
        parent_entity.insert((AudioPallet, Self { entities }));
    }
}

#[derive(Component, Clone)]
pub struct TransientAudioPallet {
    pub entities: HashMap<String, Entity>,
}

impl TransientAudioPallet {
    pub fn insert(components: Vec<(String, impl Bundle)>, parent_entity: &mut EntityCommands) {
        let entities = Self::spawn_children(components, parent_entity);
        parent_entity.insert((AudioPallet, Self { entities }));
    }

    pub fn play_transient_audio(
        commands: &mut Commands,
        entity: Entity,
        transient_audio: &TransientAudio,
    ) {
        commands.entity(entity).with_children(|parent| {
            parent.spawn(transient_audio.bundle());
        });
    }
}

trait AudioPalletSpawner {
    fn spawn_children(
        components: Vec<(String, impl Bundle)>,
        parent_entity: &mut EntityCommands,
    ) -> HashMap<String, Entity> {
        let mut entities = HashMap::new();
        parent_entity.with_children(|parent| {
            for (name, audio_component) in components {
                let entity = parent.spawn(audio_component).id();
                entities.insert(name, entity);
            }
        });
        entities
    }
}

impl AudioPalletSpawner for ContinuousAudioPallet {}
impl AudioPalletSpawner for TransientAudioPallet {}