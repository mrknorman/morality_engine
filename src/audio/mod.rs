use std::{
    collections::HashMap, 
    time::Duration
};
use rand::prelude::*;

use bevy::{
    audio::{PlaybackMode, Volume},
    ecs::component::StorageType,
    prelude::*,
};

use crate::time::Dilation;

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
        ).add_systems(
            Update,
            (
                DilatableAudio::dilate
            )
            .run_if(resource_changed::<Dilation>)
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
        source: Handle<AudioSource>,
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
            source: source,
            cooldown_timer,
            persistent,
            volume
        }
    }

    pub fn play(&self, dilation : f32) -> (AudioPlayer::<AudioSource>, PlaybackSettings) {
        (
            AudioPlayer::<AudioSource>(self.source.clone()), 
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(self.volume),
                speed : dilation,
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

pub struct ContinuousAudioPallet {
    pub entities: HashMap<String, Entity>,
    pub components: Vec<(String, AudioPlayer::<AudioSource>, PlaybackSettings, Option<DilatableAudio>)>
}

impl ContinuousAudioPallet {
    pub fn new(
        components : Vec<(String, AudioPlayer::<AudioSource>, PlaybackSettings, Option<DilatableAudio>)>
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
        hooks.on_insert(|
            mut world, 
            entity, 
            _component_id| {
        
                // Step 1: Extract components from the pallet
                let components = {
                    let mut entity_mut = world.entity_mut(
                        entity
                    );
                    entity_mut.get_mut::<ContinuousAudioPallet>()
                        .map(
                            |pallet| 
                            pallet.components.clone()
                        )
                };
        
                // Step 2: Spawn child entities and collect their IDs
                let dilation = world.get_resource::<Dilation>().map(|d| d.0);
                let mut commands = world.commands();
                let mut entities = HashMap::new();
                
                if let Some(components) = components {
                    commands.entity(entity).with_children(|parent| {
                        for (
                            name, audio_component, playback_settings, dilatable
                        ) in components.iter() {

                            let mut playback_settings = playback_settings.clone();
                            if dilatable.is_some() {
                                if let Some(dilation) = dilation {
                                    playback_settings.speed = dilation;
                                }
                            }
                            
                            let mut entity_commands = parent.spawn((
                                audio_component.clone(),
                                playback_settings
                            ));

                            if dilatable.is_some() {
                                entity_commands.insert(DilatableAudio);
                            }

                            let child_entity = entity_commands.id();

                            entities.insert(name.clone(), child_entity);
                        }
                    });
                }
        
                // Step 3: Update the pallet with the new entity map
                if let Some(
                    mut pallet
                ) = world.entity_mut(entity).get_mut::<ContinuousAudioPallet>() {
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

pub struct TransientAudioPallet {
    pub entities: HashMap<String, Vec<Entity>>,
    pub components: Vec<(String, Vec<TransientAudio>, Option<DilatableAudio>)>
}

impl TransientAudioPallet {
    pub fn new(components: Vec<(String, Vec<TransientAudio>, Option<DilatableAudio>)>) -> Self {
        Self {
            entities: HashMap::new(),
            components,
        }
    }

    pub fn play(
        commands: &mut Commands,
        entity: Entity,
        transient_audio: &mut TransientAudio,
        dilatable: bool,
        dilation: f32,
    ) {
        if !transient_audio.cooldown_timer.finished() {
            return;
        }

        let audio = if dilatable {
            transient_audio.play(dilation)
        } else {
            transient_audio.play(1.0)
        };

        if !transient_audio.persistent {
            commands.entity(entity).with_children(|parent| {
                if dilatable {
                    parent.spawn((audio, DilatableAudio));
                } else {
                    parent.spawn(audio);
                }
            });
        } else {
            if dilatable {
                commands.spawn((audio, DilatableAudio));
            } else {
                commands.spawn(audio);
            }
        }

        transient_audio.cooldown_timer.reset();
    }
    pub fn play_transient_audio(
        entity: Entity,
        commands: &mut Commands,
        pallet: &TransientAudioPallet,
        key: String,
        dilation: f32,
        audio_query: &mut Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    ) {
        let Some(audio_entity) = pallet.entities.get(&key) else {
            return;
        };

        let Some(random_sound) = audio_entity.choose(&mut thread_rng()).cloned() else {
            return;
        };

        if let Ok((mut transient_audio, dilatable)) = audio_query.get_mut(random_sound) {
            Self::play(
                commands,
                entity,
                &mut transient_audio,
                dilatable.is_some(),
                dilation,
            );
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
                    commands.entity(entity).with_children(|parent: &mut ChildBuilder<'_>| {
                        for (name, audio_components, dilatable) in components.iter() {
                            let mut child_vector = vec![];
                            for component in audio_components.iter() {

                                if dilatable.is_some() {
                                    child_vector.push(
                                        parent.spawn((
                                            component.clone(),
                                            DilatableAudio
                                        )).id()
                                    );
                                } else {
                                    child_vector.push(
                                        parent.spawn(component.clone()).id()
                                    );
                                }
      
                            }
                            entities.insert(name.clone(), child_vector);
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
                
                if let Some(entities) = entities {
                    let mut commands = world.commands();
                    for (_name, child_entities) in entities {
                        for child_entity in child_entities {
                            if commands.get_entity(child_entity).is_some() {
                                commands.entity(child_entity).despawn_recursive();
                            }
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

pub struct OneShotAudioPallet {
    pub components: Vec<(OneShotAudio, Option<DilatableAudio>)>
}

impl OneShotAudioPallet {
    pub fn new(
        components : Vec<(OneShotAudio, Option<DilatableAudio>)>
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
                let dilation = world.get_resource::<Dilation>().map(|d| d.0);
                let mut commands = world.commands();
                if let Some(components) = components {
                    
                    for (audio_component, dilatable) in components.iter() {

                        if !audio_component.persistent {
                            commands.entity(entity).with_children(
                                |parent| {

                                    let mut entity_commands = parent.spawn((
                                        AudioPlayer::<AudioSource>(audio_component.source.clone()),
                                        PlaybackSettings {
                                            paused: false,
                                            mode: PlaybackMode::Despawn,
                                            volume: Volume::new(audio_component.volume),
                                            speed: dilation.filter(|_| dilatable.is_some()).unwrap_or(1.0),
                                            ..default()
                                        }
                                    ));
                                    
                                    if dilatable.is_some() {
                                        entity_commands.insert(DilatableAudio);
                                    } 
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

#[derive(Component, Clone)]
pub struct DilatableAudio;
impl DilatableAudio {
    fn dilate(
        dilation : Res<Dilation>,
        mut audio_query : Query<&mut AudioSink, With<Self>>
    ) {
        for audio in audio_query.iter_mut() {
            audio.set_speed(dilation.0);
        }
    }
}