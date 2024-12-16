use std::{
    collections::HashMap, 
    time::Duration
};

use bevy::{
    asset::AssetPath,
    audio::{PlaybackMode, Volume},
    ecs::component::StorageType,
    prelude::*,
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioSystemsActive {
    #[default]
    False,
    True
}

pub struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<AudioSystemsActive>()
        .add_event::<NarrationAudioFinished>()
        .insert_resource(
            MusicAudioConfig::new(1.0)
        )
        .insert_resource(
            NarrationAudioConfig::new(1.0)
        )
        .add_systems(
            Update,
            activate_systems
        ).add_systems(
            Update,
            (
                NarrationAudio::check_if_finished,
                TransientAudio::tick
            )
            .run_if(
                in_state(AudioSystemsActive::True)
            )
        );
    }
}

fn activate_systems(
	mut audio_state: ResMut<NextState<AudioSystemsActive>>,
	transient_query: Query<&TransientAudio>,
) {
	if !transient_query.is_empty() {
		audio_state.set(AudioSystemsActive::True)
	} else {
		audio_state.set(AudioSystemsActive::False)
	}
}

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
            AudioPlayer::<AudioSource>(asset_server.load(audio_path)),
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(0.5),
                ..default()
            }
        )).id()
}

pub fn continuous_audio() -> PlaybackSettings {
    PlaybackSettings {
        paused : false,
        mode: PlaybackMode::Loop,
        ..default()
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

    pub fn play(&self) -> (AudioPlayer::<AudioSource>, PlaybackSettings) {
        (
            AudioPlayer::<AudioSource>(self.source.clone()), 
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(self.volume),
                ..default()
            }
        )
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
    pub components: Vec<(String, AudioPlayer::<AudioSource>, PlaybackSettings)>
}

impl ContinuousAudioPallet {
    pub fn new(
        components : Vec<(String, AudioPlayer::<AudioSource>, PlaybackSettings)>
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
                        .map(
                            |pallet| 
                            pallet.components.clone()
                        )
                };
        
                // Step 2: Spawn child entities and collect their IDs
                let mut commands = world.commands();
                let mut entities = HashMap::new();
                
                if let Some(components) = components {
                    commands.entity(entity).with_children(|parent| {
                        for (
                            name, audio_component, playback_settings
                        ) in components.iter() {
                            
                            let child_entity = parent.spawn((
                                audio_component.clone(),
                                *playback_settings
                            )).id();
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

#[derive(Component, Clone)]
pub struct OneShotAudio {
    pub source: Handle<AudioSource>,
    pub persistent : bool,
    pub volume: f32
}

pub fn one_shot_audio() -> PlaybackSettings {
    PlaybackSettings {
        paused : false,
        mode: PlaybackMode::Despawn,
        ..default()
    }
}

#[derive(Clone)]
pub struct OneShotAudioPallet {
    pub components: Vec<OneShotAudio>
}

impl OneShotAudioPallet {
    pub fn new(
        components : Vec<OneShotAudio>
    ) -> Self {
        Self {
            components
        }
    }
}

impl Component for OneShotAudioPallet {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
        
                // Step 1: Extract components from the pallet
                let components = {
                    let mut entity_mut = world.entity_mut(entity);
                    entity_mut.get_mut::<OneShotAudioPallet>()
                        .map(|pallet| pallet.components.clone())
                };
                // Step 2: Spawn child entities and collect their IDs
                let mut commands = world.commands();
                if let Some(components) = components {
                    
                    for audio_component in components.iter() {

                        if !audio_component.persistent {
                            commands.entity(entity).with_children(
                                |parent| {
                                    parent.spawn(
                                        (
                                            AudioPlayer::<AudioSource>(audio_component.source.clone()), 
                                            PlaybackSettings {
                                                paused : false,
                                                mode: PlaybackMode::Despawn,
                                                volume: Volume::new(
                                                    audio_component.volume
                                                ),
                                                ..default()
                                            }
                                        )
                                    );
                                }
                            );
                        } else {
                            commands.spawn(
                                (
                                    AudioPlayer::<AudioSource>(audio_component.source.clone()), 
                                    PlaybackSettings {
                                        paused : false,
                                        mode: PlaybackMode::Despawn,
                                        volume: Volume::new(
                                            audio_component.volume
                                        ),
                                        ..default()
                                    }
                                )
                            );
                        }
                    } 
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

// Define the AudioLayer trait with a volume field
trait AudioLayer {
    fn volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);
}

pub struct MusicAudio;

impl Component for MusicAudio {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {            

                // Get an immutable reference to the resource
                let mut previous_entity : Option<Entity> = None;
                if let Some(audio_config) = world.get_resource::<MusicAudioConfig>() {
                    if let Some(entity) = audio_config.entity {
                        if world.get_entity(entity).is_ok() {
                            previous_entity = Some(entity);
                        }
                    }
                }

                let mut commands = world.commands();
                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn_recursive();
                }
                
                if let Some(mut audio_config) = world.get_resource_mut::<MusicAudioConfig>() {
                    audio_config.entity = Some(entity);
                }
            }
        );
    }
}

#[derive(Resource)]
struct MusicAudioConfig {
    volume: f32,
    entity : Option<Entity>
}

impl MusicAudioConfig {
    fn new(volume: f32) -> Self {
        Self { 
            volume,
            entity : None
        }
    }
}

impl AudioLayer for MusicAudioConfig {
    fn volume(&self) -> f32 {
        self.volume
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

#[derive(Event)]
pub struct NarrationAudioFinished;

pub struct NarrationAudio;

impl NarrationAudio {
    fn check_if_finished(  
        mut narration_query : Query<&AudioSink, With<NarrationAudio>>,
        mut ev_narration_finished: EventWriter<NarrationAudioFinished>,
    ) {

        for audio in narration_query.iter_mut() {
            if audio.empty() {
                ev_narration_finished.send(NarrationAudioFinished);
            }
        }
    }
}

impl Component for NarrationAudio {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {            

                // Get an immutable reference to the resource
                let mut previous_entity : Option<Entity> = None;
                if let Some(audio_config) = world.get_resource::<NarrationAudioConfig>() {
                    if let Some(entity) = audio_config.entity {
                        if world.get_entity(entity).is_ok() {
                            previous_entity = Some(entity);
                        }
                    }
                }

                let mut commands = world.commands();
                if let Some(entity) = previous_entity {
                    commands.entity(entity).despawn_recursive();
                }
                
                if let Some(mut audio_config) = world.get_resource_mut::<NarrationAudioConfig>() {
                    audio_config.entity = Some(entity);
                }
            }
        );
    }
}

// Define the Narration component
#[derive(Resource)]
pub struct NarrationAudioConfig {
    volume: f32,
    entity : Option<Entity>,
}

impl NarrationAudioConfig {
    fn new(volume: f32) -> Self {
        Self {             
            volume,
            entity : None
        }
    }
}

impl AudioLayer for NarrationAudioConfig {
    fn volume(&self) -> f32 {
        self.volume
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

#[derive(Component)]
pub struct BackgroundAudio;

// Define the Background component
#[derive(Component)]
struct BackgroundAudioConfig {
    volume: f32
}

impl BackgroundAudioConfig {
    fn new(volume: f32) -> Self {
        Self { 
            volume
        }
    }
}

impl AudioLayer for BackgroundAudioConfig {
    fn volume(&self) -> f32 {
        self.volume
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}

#[derive(Component)]
pub struct EffectAudio;

#[derive(Component)]
struct EffectAudioConfig {
    volume: f32
}

impl EffectAudioConfig {
    fn new(volume: f32) -> Self {
        Self { 
            volume
        }
    }
}

impl AudioLayer for EffectAudioConfig {
    fn volume(&self) -> f32 {
        self.volume
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }
}