use std::f32::consts::FRAC_PI_4;

use bevy::{
	prelude::*,
	ecs::component::StorageType,
	audio::Volume
};
use enum_map::{
    Enum, 
    enum_map
};
use rand::Rng;
use serde::Deserialize;


pub mod content;
use content::TrainTypes;

use crate::{
    data::rng::GlobalRng,
	systems::{
        audio::{
            continuous_audio, 
            AudioPlugin, 
            ContinuousAudio, 
            ContinuousAudioPallet, 
            OneShotAudio, 
            OneShotAudioPallet, 
            TransientAudio, 
            TransientAudioPallet
        }, 
        interaction::{
            ActionPallet, 
            Clickable, 
            InputAction, 
            InteractionPlugin
        },
	    colors::ColorAnchor, 
        motion::Wobble, 
        physics::Velocity
    },
    entities::text::{
		Animated, 
		TextFrames, 
		TextSprite
	}
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrainSystemsActive {
    #[default]
	False,
    True
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainActions {
    TriggerHorn
}

impl std::fmt::Display for TrainActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct TrainPlugin;
impl Plugin for TrainPlugin {
    fn build(&self, app: &mut App) {	
		if !app.is_plugin_added::<InteractionPlugin>() {
			app.add_plugins(InteractionPlugin);
		};

		if !app.is_plugin_added::<AudioPlugin>() {
			app.add_plugins(AudioPlugin);
		};

		app
		.init_state::<TrainSystemsActive>()
		.add_systems(
			Update,
			activate_systems
		).add_systems(
            Update,
            (
				Animated::animate_text
            )
            .run_if(in_state(TrainSystemsActive::True))
        )
		.register_required_components::<Train, Velocity>()
		.register_required_components::<Train, Transform>()
        .register_required_components::<Train, Visibility>()
		.register_required_components::<Train, TrainState>()
		.register_required_components::<Train, TextSprite>();
    }
}

fn activate_systems(
	mut train_state: ResMut<NextState<TrainSystemsActive>>,
	train_query: Query<&Train>
) {
	if !train_query.is_empty() {
		train_state.set(TrainSystemsActive::True)
	} else {
		train_state.set(TrainSystemsActive::False)
	}
}

#[derive(Deserialize, Clone)]
pub struct TrainType{
    pub carriages: Vec<String>,
    pub smoke: Option<Vec<String>>,
	pub track_audio_path: String,
	pub stopped_audio_path : String,
	pub horn_audio_path: Option<String>,
	pub rising_smoke: Option<Vec<String>>
}

impl TrainType {
    pub fn load_from_json(train_type : TrainTypes) -> TrainType {


        let train_type: TrainType = serde_json::from_str(
			train_type.content()
		).unwrap_or_else(|err| {
            panic!("Failed to parse train from file {}.", err);
        });

        // Additional validation
        if train_type.carriages.is_empty() {
            panic!("TrainType must have at least one carriage");
        }

        train_type
    }
}

#[derive(Component)]
#[require(TextSprite, Text2d)]
pub struct TrainCarriage;

#[derive(Component)]
#[require(Animated)]
pub struct TrainSmoke;

#[derive(Clone)]
pub struct Train(pub TrainTypes);

#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
pub enum TrainState {
	#[default]
	Moving, 
	Wrecked
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainSounds {
	Tracks,
	Horn
}

impl Component for Train {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _| {
                // Get GlobalRng first, before other borrows
                let mut rng_option = None;
                if let Some(rng) = world.get_resource_mut::<GlobalRng>() {
                    rng_option = Some(rng.0.clone()); // Clone the Rng so we can use it without holding the borrow
                }
                
                // Now collect other data
                let (carriages, smoke_frames, burning_frames, horn_audio, track_audio, hiss_audio, burning_audio, color, train_state) = {
                    if let (Some(train), Some(color), Some(train_state)) = (
                        world.entity(entity).get::<Train>(),
                        world.entity(entity).get::<TextColor>(),
                        world.entity(entity).get::<TrainState>()
                    ) {
                        let train_type = TrainType::load_from_json(train.0);
                       
                        let carriages = train_type.carriages.clone();
                        let smoke_frames: Vec<String> = train_type.clone().smoke.unwrap_or(vec![]);
						let burning_frames: Vec<String> = train_type.clone().rising_smoke.unwrap_or(vec![]);
                        let asset_server = world.get_resource::<AssetServer>().unwrap();
                        let horn_audio: Option<TransientAudio> = train_type.clone().horn_audio_path.map(
                            |path| {
                                TransientAudio::new(
                                    asset_server.load(path),
                                    2.0,
                                    false,
                                    1.0,
                                    true
                                )
                        });
                        let track_audio: Handle<AudioSource> = asset_server.load(train_type.track_audio_path);

						let hiss_audio = asset_server.load(train_type.stopped_audio_path);
						let burning_audio = asset_server.load("./audio/effects/train/fire.ogg");

                        (
                            carriages, 
                            smoke_frames, 
							burning_frames,
                            horn_audio, 
                            track_audio, 
							hiss_audio,
							burning_audio,
                            *color,
                            *train_state
                        )
                    } else {
                        panic!("Train unable to spawn!");
                    }
                };
                
                // Continue only if we got the RNG
                let mut rng = match rng_option {
                    Some(rng) => rng,
                    None => panic!("Rng not found!"),
                };
                
                // Now get the commands after all data is collected
                let mut commands = world.commands();
                
                // Use the commands to modify entities
                let mut carriage_entities: Vec<Entity> = vec![];

                if train_state == TrainState::Moving {
                    commands.entity(entity).insert(
                        ContinuousAudioPallet::new(
                            vec![
                                ContinuousAudio{
                                    key: TrainSounds::Tracks,
                                    source: AudioPlayer::<AudioSource>(track_audio),
                                    settings: PlaybackSettings{
                                        volume: Volume::new(0.1),
                                        ..continuous_audio()
                                    },
                                    dilatable: true
                                }
                            ]
                        )
                    );
                } else if train_state == TrainState::Wrecked {

					commands.entity(entity).insert((
						OneShotAudioPallet::new(
							vec![
								OneShotAudio{
									source: hiss_audio,
									volume : 0.4,
									persistent : false,
									dilatable: true
								}
							]
						),
                        ContinuousAudioPallet::new(
                            vec![
                                ContinuousAudio{
                                    key: TrainSounds::Tracks,
                                    source: AudioPlayer::<AudioSource>(burning_audio),
                                    settings: PlaybackSettings{
                                        volume: Volume::new(0.4),
                                        ..continuous_audio()
                                    },
                                    dilatable: true
                                }
                            ]
                        ))
                    );
				}
                
                commands.entity(entity).with_children(
                    |parent| {
                    let mut carriage_translation = Vec3::default();
                    for carriage in carriages.clone() {
                        let mut transform = Transform::from_translation(carriage_translation);
                        
                        if train_state == TrainState::Wrecked {
                            // Add random rotation between 0 and 20 degrees for wrecked carriages (except first)
                            let random_rotation = rng.gen_range(-FRAC_PI_4..FRAC_PI_4);
                            transform.rotate(Quat::from_rotation_z(random_rotation));
                        }
                        
                        let mut entity = parent.spawn((
                            TrainCarriage,
                            Text2d::new(carriage),
                            transform,
                            color
                        ));
                        
                        carriage_entities.push(entity.id());

						if train_state == TrainState::Moving {
							entity.insert(Wobble::default());
						}
                        
                        // Update translation for next carriage
                        if train_state == TrainState::Wrecked {
                            // Vary the distance for wrecked trains (except first position)
                            carriage_translation.x -= rng.gen_range(70.0..100.0);
                        } else {
                            carriage_translation.x -= 85.0;
                        }
                    }

					if train_state != TrainState::Wrecked {
						parent.spawn((
							TrainSmoke,
							Text2d(smoke_frames[0].clone()),
							TextFrames::new(smoke_frames),
							Transform::from_xyz(
								-35.0,
								32.0,
								0.1
							),
							color,
							ColorAnchor(color.0)
						));
					} else if train_state == TrainState::Wrecked {
						// Clone once for creating the TextFrames component.
						let text_frames = TextFrames::new(burning_frames.clone());

						// Define the X positions and the corresponding indices in burning_frames.
						let spawn_data = [
							(-35.0, 0),
							(0.0, 3),
							(  35.0, 6),
						];

						for (x, idx) in spawn_data {
							// Clone only the specific frame for this spawn.
							let frame = burning_frames[idx].clone();
							parent.spawn((
								Animated::new(idx, 0.3),
								Text2d(frame),
								// Clone the text_frames for each spawn if needed.
								text_frames.clone(),
								Transform::from_xyz(x, 100.0, 0.1),
								color,
								ColorAnchor(color.0),
							));
						}
					}
                });
                
                if train_state != TrainState::Wrecked {
                    if let Some(horn_audio) = horn_audio {
                        let mut engine = commands.entity(
                            carriage_entities[0]
                        );
            
                        engine.insert((
                            Clickable::new(
                                vec![
                                    TrainActions::TriggerHorn
                                ]
                            ),
                            ActionPallet::<TrainActions, TrainSounds>(
                                enum_map!(
                                    TrainActions::TriggerHorn => vec![
                                        InputAction::PlaySound(
                                            TrainSounds::Horn
                                        )
                                    ]
                                )
                            ),
                            TransientAudioPallet::new(
                                vec![(
                                    TrainSounds::Horn,
                                    vec![horn_audio]
                                )]
                            ))
                        );
                    };
                }
            }
        );
    }
}