use std::time::Duration;

use enum_map::{
    Enum, 
    EnumArray,
    EnumMap
};
use rand::{prelude::*, rng};
use bevy::{
    audio::{
        PlaybackMode, 
        Volume
    },
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*
};

use crate::systems::time::Dilation;

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
        .add_message::<NarrationAudioFinished>()
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
    volume: f32,
    dilatable : bool
}

impl TransientAudio {
    pub fn new(
        source: Handle<AudioSource>,
        cooldown_time_seconds: f32,
        persistent : bool,
        volume: f32,
        dilatable : bool
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
            volume,
            dilatable
        }
    }

    pub fn play(&self, dilation : f32) -> (AudioPlayer::<AudioSource>, PlaybackSettings) {
        (
            AudioPlayer::<AudioSource>(self.source.clone()), 
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::Linear(self.volume),
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

#[derive(Clone)]
pub struct ContinuousAudio<T> where
T: Enum + EnumArray<Entity> + Send + Sync + Clone,
<T as EnumArray<Entity>>::Array: Send + Sync + Clone {
    pub key : T,
    pub source : AudioPlayer::<AudioSource>,
    pub settings : PlaybackSettings,
    pub dilatable : bool
}

#[derive(Component)]
#[component(on_insert = ContinuousAudioPallet::<T>::on_insert, on_remove = ContinuousAudioPallet::<T>::on_remove)]
pub struct ContinuousAudioPallet<T> where
T: Enum + EnumArray<Entity> + Send + Sync + Clone,
<T as EnumArray<Entity>>::Array: Send + Sync + Clone
{  
    pub entities : EnumMap::<T, Entity>,
    pub components: Vec<ContinuousAudio<T>>
}

impl<T> ContinuousAudioPallet<T> where
T: Enum + EnumArray<Entity> + Send + Sync + Clone + 'static,
<T as EnumArray<Entity>>::Array: Send + Sync + Clone
{
    pub fn new(
        components : Vec<ContinuousAudio<T>>
    ) -> ContinuousAudioPallet<T> {
        ContinuousAudioPallet::<T> {
            entities : enum_map::enum_map! { _ => Entity::PLACEHOLDER },
            components
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let components = {
            world.entity_mut(entity)
                .get_mut::<ContinuousAudioPallet<T>>()
                .map(|pallet| pallet.components.clone())
        };
    
        let dilation = world.get_resource::<Dilation>().map(|d| d.0);
        let mut entities = enum_map::enum_map! { _ => Entity::PLACEHOLDER };
    
        // Spawn child entities if components exist.
        if let Some(components) = components {
            world.commands().entity(entity).with_children(|parent| {
                for component in components.iter() {
                    let mut playback_settings = component.settings.clone();
                    if component.dilatable {
                        if let Some(d) = dilation {
                            playback_settings.speed = d;
                        }
                    }
    
                    let mut child = parent.spawn((component.source.clone(), playback_settings));
                    if component.dilatable {
                        child.insert(DilatableAudio);
                    }
                    entities[component.key.clone()] = child.id();
                }
            });
        }
    
        // Update the pallet with the new entity map.
        if let Some(mut pallet) = world.entity_mut(entity).get_mut::<ContinuousAudioPallet<T>>() {
            pallet.entities = entities;
        }
    
    }

    fn on_remove(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
         let entities = {
            let mut entity_mut = world.entity_mut(entity);
            entity_mut.get_mut::<ContinuousAudioPallet<T>>()
                .map(|pallet| pallet.entities.clone())
        };

        // Step 2: Attempt to despawn each child entity
        if let Some(entities) = entities {
            let mut commands = world.commands();
            for (_name, child_entity) in entities {
                // Attempt to despawn the entity, this will silently fail if the entity doesn't exist
                if commands.get_entity(child_entity).is_ok() {
                    commands.entity(child_entity).despawn();
                }
            }
        }
    }
}


#[derive(Component)]
#[component(on_insert = TransientAudioPallet::<T>::on_insert)]
pub struct TransientAudioPallet<T> where
T: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
<T as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone
{  
    pub entities : EnumMap::<T, Vec<Entity>>,
    pub components: Vec<(T, Vec<TransientAudio>)>
}

impl<T> TransientAudioPallet<T> where
T: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone,
<T as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone
{
    pub fn new(
        components : Vec<(T, Vec<TransientAudio>)>
    ) -> Self {
        TransientAudioPallet::<T> {
            entities : enum_map::enum_map! { _ => vec![]},
            components
        }
    }

    pub fn play(
        commands: &mut Commands,
        entity: Entity,
        transient_audio: &mut TransientAudio,
        dilatable: bool,
        dilation: f32,
    ) {
        if !transient_audio.cooldown_timer.is_finished() {
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
        pallet: &TransientAudioPallet<T>,
        key: T,
        dilation: f32,
        audio_query: &mut Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    ) {

        let Some(random_sound) = pallet.entities[key].choose(&mut rng()).cloned() else {
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

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        // Step 1: Extract components from the pallet
        let components = {
            let mut entity_mut = world.entity_mut(entity);
            entity_mut.get_mut::<TransientAudioPallet<T>>()
                .map(|pallet| pallet.components.clone())
        };

        // Step 2: Spawn child entities and collect their IDs
        let mut commands = world.commands();
        let mut entities = enum_map::enum_map! { _ => vec![] };
        
        if let Some(components) = components {
            commands.entity(entity).with_children(|parent: &mut ChildSpawnerCommands<'_>| {
                for (key, audio_components) in components.iter() {
                    let mut child_vector = vec![];
                    for component in audio_components.iter() {
                        if component.dilatable {
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
                    entities[key.clone()] = child_vector;
                }
            });
        }

        // Step 3: Update the pallet with the new entity map
        if let Some(mut pallet) = world.entity_mut(entity).get_mut::<TransientAudioPallet<T>>() {
            pallet.entities = entities;
        }
    }
}

#[derive(Component, Clone)]
pub struct OneShotAudio {
    pub source: Handle<AudioSource>,
    pub persistent : bool,
    pub volume: f32,
    pub dilatable : bool,
    pub speed : f32
}

impl Default for OneShotAudio{
    fn default() -> Self {
        Self {
            source: Handle::default(),
            persistent: false,
            volume: 1.0,
            dilatable: false,
            speed : 1.0
        }
    }
}

pub fn one_shot_audio() -> PlaybackSettings {
    PlaybackSettings {
        paused : false,
        mode: PlaybackMode::Despawn,
        ..default()
    }
}

#[derive(Component)]
#[component(on_insert = OneShotAudioPallet::on_insert)]
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

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {

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
            
            for audio_component in components.iter() {

                if !audio_component.persistent {
                    commands.entity(entity).with_children(
                        |parent| {

                            let mut entity_commands = parent.spawn((
                                AudioPlayer::<AudioSource>(audio_component.source.clone()),
                                PlaybackSettings {
                                    paused: false,
                                    mode: PlaybackMode::Despawn,
                                    volume: Volume::Linear(audio_component.volume),
                                    speed: dilation.filter(|_| audio_component.dilatable).unwrap_or(audio_component.speed),
                                    ..default()
                                }
                            ));
                            
                            if audio_component.dilatable {
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
                                volume: Volume::Linear(
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
}
// Define the AudioLayer trait with a volume field


trait AudioLayer {
    fn volume(&self) -> f32;
    fn set_volume(&mut self, volume: f32);
}

#[derive(Component)]
#[component(on_insert = MusicAudioConfig::on_insert)]
pub struct MusicAudio;

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

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
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
              commands.entity(entity).despawn();
          }
          
          if let Some(mut audio_config) = world.get_resource_mut::<MusicAudioConfig>() {
              audio_config.entity = Some(entity);
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

#[derive(Message)]
pub struct NarrationAudioFinished;

#[derive(Component)]
#[component(on_insert = NarrationAudio::on_insert)]
pub struct NarrationAudio;
impl NarrationAudio {
    fn check_if_finished(  
        mut narration_query : Query<&AudioSink, With<NarrationAudio>>,
        mut ev_narration_finished: MessageWriter<NarrationAudioFinished>,
    ) {

        for audio in narration_query.iter_mut() {
            if audio.empty() {
                ev_narration_finished.write(NarrationAudioFinished);
            }
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
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
              commands.entity(entity).despawn();
          }
          
          if let Some(mut audio_config) = world.get_resource_mut::<NarrationAudioConfig>() {
              audio_config.entity = Some(entity);
          }
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

#[derive(Component)]
pub struct DialogueAudio;