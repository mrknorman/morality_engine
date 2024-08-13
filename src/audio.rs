use bevy::{
    asset::AssetPath,
    audio::{PlaybackMode, Volume},
    ecs::system::EntityCommands,
    prelude::*,
};
use std::{collections::HashMap, time::Duration};

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
    cooldown_timer: Timer,
    volume: f32
}

impl TransientAudio {
    pub fn new(
        audio_path: impl Into<AssetPath<'static>>,
        asset_server: &Res<AssetServer>,
        cooldown_time_seconds: f32,
        volume: f32,
    ) -> Self {

        let mut cooldown_timer = Timer::from_seconds(
            cooldown_time_seconds,
            TimerMode::Once
        );
        cooldown_timer.tick(
            Duration::from_secs_f32(
                cooldown_time_seconds
            )
        );
        Self {
            source: asset_server.load(audio_path),
            cooldown_timer,
            volume
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

    pub fn tick(
        time : Res<Time>,
        mut audio_query : Query<&mut TransientAudio>
    ) {

        for mut audio in audio_query.iter_mut() {
            audio.cooldown_timer.tick(time.delta());
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
        transient_audio: &mut TransientAudio
    ) {

        if transient_audio.cooldown_timer.finished() {
            commands.entity(entity).with_children(|parent| {
                parent.spawn(transient_audio.bundle());
            });

            transient_audio.cooldown_timer.reset();
        }
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

pub struct AudioPlugin<T: States + Clone + Eq + Default> {
    active_state: T,
}


impl<T: States + Clone + Eq + Default> AudioPlugin<T> {
    pub fn new(active_state: T) -> Self {
        Self { active_state }
    }
}

impl<T: States + Clone + Eq + Default + 'static> Plugin for AudioPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                TransientAudio::tick
            )
            .run_if(in_state(self.active_state.clone()))
        );
    }
}