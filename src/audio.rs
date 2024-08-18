use bevy::{
    asset::AssetPath,
    audio::{PlaybackMode, Volume},
    ecs::component::StorageType,
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
pub struct ContinuousAudio {
    source: Handle<AudioSource>,
    volume: f32,
}

impl ContinuousAudio {
    pub fn new(
        asset_server: &Res<AssetServer>,
        audio_path: impl Into<AssetPath<'static>>,
        volume: f32,
    ) -> ContinuousAudio {

        ContinuousAudio {
            source: asset_server.load(audio_path),
            volume
        }
    }
}

#[derive(Bundle)]
struct ContinuousAudioBundle {
    audio : AudioBundle,
    continuous_audio : ContinuousAudio
}

impl ContinuousAudioBundle {

    fn from_continuous_audio(
        continuous_audio : ContinuousAudio
    ) -> Self {

        Self {
            audio : AudioBundle {
                source: continuous_audio.clone().source,
                settings: PlaybackSettings {
                    mode: PlaybackMode::Loop,
                    volume: Volume::new(continuous_audio.clone().volume),
                    ..default()
                },
            },
            continuous_audio
        }
    }
}



#[derive(Component, Clone)]
pub struct TransientAudio {
    source: Handle<AudioSource>,
    cooldown_timer: Timer,
    persistent : bool,
    volume: f32
}

impl TransientAudio {
    pub fn new(
        audio_path: impl Into<AssetPath<'static>>,
        asset_server: &Res<AssetServer>,
        cooldown_time_seconds: f32,
        persistent : bool,
        volume: f32,
    ) -> TransientAudio {

        let mut cooldown_timer = Timer::from_seconds(
            cooldown_time_seconds,
            TimerMode::Once
        );
        cooldown_timer.tick(
            Duration::from_secs_f32(
                cooldown_time_seconds
            )
        );
        TransientAudio {
            source: asset_server.load(audio_path),
            cooldown_timer,
            persistent,
            volume
        }
    }

    pub fn play(&self) -> AudioBundle {
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

#[derive(Clone)]
pub struct ContinuousAudioPallet {
    pub entities: HashMap<String, Entity>,
    pub components: Vec<(String, ContinuousAudio)>
}

impl ContinuousAudioPallet {
    pub fn new(
        components : Vec<(String, ContinuousAudio)>
    ) -> ContinuousAudioPallet {
        ContinuousAudioPallet {
            entities : HashMap::new(),
            components
        }
    }
}

impl Component for ContinuousAudioPallet {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
        
                // Step 1: Extract components from the pallet
                let components = {
                    let mut entity_mut = world.entity_mut(entity);
                    entity_mut.get_mut::<ContinuousAudioPallet>()
                        .map(|pallet| pallet.components.clone())
                };
        
                // Step 2: Spawn child entities and collect their IDs
                let mut commands = world.commands();
                let mut entities = HashMap::new();
                
                if let Some(components) = components {
                    commands.entity(entity).with_children(|parent| {
                        for (name, audio_component) in components.iter() {
                            let child_entity = parent.spawn(
                                ContinuousAudioBundle::from_continuous_audio(audio_component.clone())
                            ).id();
                            entities.insert(name.clone(), child_entity);
                        }
                    });
                }
        
                // Step 3: Update the pallet with the new entity map
                if let Some(mut pallet) = world.entity_mut(entity).get_mut::<ContinuousAudioPallet>() {
                    pallet.entities = entities;
                }
            }
        );
        hooks.on_remove(
            |mut world, entity, _component_id| {
                // Step 1: Extract the entity map from the pallet
                let entities = {
                    let mut entity_mut = world.entity_mut(entity);
                    entity_mut.get_mut::<ContinuousAudioPallet>()
                        .map(|pallet| pallet.entities.clone())
                };
        
                // Step 2: Attempt to despawn each child entity
                if let Some(entities) = entities {
                    let mut commands = world.commands();
                    for (_name, child_entity) in entities {
                        // Attempt to despawn the entity, this will silently fail if the entity doesn't exist
                        if commands.get_entity(child_entity).is_some() {
                            commands.entity(child_entity).despawn_recursive();
                        }
                    }
                }
            }
        );
    }
}

#[derive(Clone)]
pub struct TransientAudioPallet {
    pub entities: HashMap<String, Entity>,
    pub components: Vec<(String, TransientAudio)>
}

impl TransientAudioPallet {
    pub fn new(
        components : Vec<(String, TransientAudio)>
    ) -> TransientAudioPallet {
        TransientAudioPallet {
            entities : HashMap::new(),
            components
        }
    }

    pub fn play_transient_audio(
        commands: &mut Commands,
        entity: Entity,
        transient_audio: &mut TransientAudio
    ) {
        if transient_audio.cooldown_timer.finished() {

            if !transient_audio.persistent {
                commands.entity(entity).with_children(|parent| {
                    parent.spawn(transient_audio.play());
                });
            } else {
                commands.spawn(transient_audio.play());
            }

            transient_audio.cooldown_timer.reset();
        }
    }

}

impl Component for TransientAudioPallet {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
        
                // Step 1: Extract components from the pallet
                let components = {
                    let mut entity_mut = world.entity_mut(entity);
                    entity_mut.get_mut::<TransientAudioPallet>()
                        .map(|pallet| pallet.components.clone())
                };
        
                // Step 2: Spawn child entities and collect their IDs
                let mut commands = world.commands();
                let mut entities = HashMap::new();
                
                if let Some(components) = components {
                    commands.entity(entity).with_children(|parent| {
                        for (name, audio_component) in components.iter() {
                            let child_entity = parent.spawn(audio_component.clone()).id();
                            entities.insert(name.clone(), child_entity);
                        }
                    });
                }
        
                // Step 3: Update the pallet with the new entity map
                if let Some(mut pallet) = world.entity_mut(entity).get_mut::<TransientAudioPallet>() {
                    pallet.entities = entities;
                }
            }
        );
        hooks.on_remove(
            |mut world, entity, _component_id| {
                // Step 1: Extract the entity map from the pallet
                let entities = {
                    let mut entity_mut = world.entity_mut(entity);
                    entity_mut.get_mut::<TransientAudioPallet>()
                        .map(|pallet| pallet.entities.clone())
                };
        
                // Step 2: Attempt to despawn each child entity
                if let Some(entities) = entities {
                    let mut commands = world.commands();
                    for (_name, child_entity) in entities {
                        // Attempt to despawn the entity, this will silently fail if the entity doesn't exist
                        if commands.get_entity(child_entity).is_some() {
                            commands.entity(child_entity).despawn_recursive();
                        }
                    }
                }
            }
        );
    }
}

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