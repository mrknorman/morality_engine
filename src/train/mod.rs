use std::{
	fs::File,
	io::BufReader
};

use bevy::{
	prelude::*,
	ecs::component::StorageType
};
use serde::Deserialize;

use crate::{
    audio::{
		AudioPlugin, ContinuousAudioPallet, ContinuousAudio, TransientAudio, TransientAudioPallet
	}, 
	interaction::{
		Clickable, InputAction, InteractionPlugin
	}, motion::{
		Locomotion, 
		TranslationAnchor, 
		Wobble
	}, 
	text::{
		AnimatedTextSpriteBundle, 
		TextSpriteBundle
	}
};

pub static STEAM_TRAIN: &str = "text/trains/steam_train.json";

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrainSystemsActive {
    #[default]
	False,
    True
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
				AnimatedTextSpriteBundle::animate,
                Wobble::wobble,
				Locomotion::locomote,
            )
            .run_if(in_state(TrainSystemsActive::True))
        );
    }
}

fn activate_systems(
	mut interaction_state: ResMut<NextState<TrainSystemsActive>>,
	train_query: Query<&Train>
) {
	if !train_query.is_empty() {
		interaction_state.set(TrainSystemsActive::True)
	} else {
		interaction_state.set(TrainSystemsActive::False)
	}
}

#[derive(Deserialize)]
pub struct TrainType{
    pub carriages: Vec<String>,
    pub smoke: Option<Vec<String>>,
	pub track_audio_path: String,
	pub horn_audio_path: Option<String>
}

impl TrainType {
    pub fn load_from_json(file_path: String) -> TrainType {
        let file = File::open(&file_path).unwrap_or_else(|err| {
            panic!("Failed to open file {}: {}", file_path, err);
        });

        let reader = BufReader::new(file);
        let train_type: TrainType = serde_json::from_reader(
			reader
		).unwrap_or_else(|err| {
            panic!("Failed to parse JSON from file {}: {}", file_path, err);
        });

        // Additional validation
        if train_type.carriages.is_empty() {
            panic!("TrainType must have at least one carriage");
        }

        train_type
    }
}

#[derive(Component, Clone)]
pub struct TrainCarriage;

#[derive(Bundle, Clone)]
pub struct TrainCarriageBundle{
	marker : TrainCarriage,
	anchor : TranslationAnchor,
	wobble : Wobble,
	sprite : TextSpriteBundle
}

impl TrainCarriageBundle{

	pub fn new(
		text : String,
		translation : Vec3
	) -> Self {

		Self {
			marker : TrainCarriage,
			anchor : TranslationAnchor::new(
				translation
			),
			wobble : Wobble::new(),
			sprite : TextSpriteBundle::new(
				text,
				translation
			)
		}
	}
}

#[derive(Component, Clone)]
pub struct TrainSmoke;

#[derive(Bundle, Clone)]
pub struct TrainSmokeBundle{
	marker : TrainSmoke,
	animation_sprite : AnimatedTextSpriteBundle
}

impl TrainSmokeBundle {

	pub fn new(
			frames : Vec<String>, 
			translation : Vec3
		) -> Self {

		Self {
			marker : TrainSmoke,
			animation_sprite : AnimatedTextSpriteBundle::from_vec(
				frames.iter().map(
					|s| s.to_string()
				).collect(),
				0.1,
				Vec3::new(
					translation.x - 35.0,
					translation.y + 32.0,
					translation.z,
				)
			)
		}
	}
}

#[derive(Clone)]
pub struct Train {
	pub carriages : Vec<TrainCarriageBundle>,
	pub smoke : TrainSmokeBundle,
	pub horn_audio : Option<TransientAudio>
}

#[derive(Clone, Bundle)]
pub struct TrainBundle {
	train : Train,
	locomotion : Locomotion,
	transform : Transform,
	visibility : Visibility,
	audio : ContinuousAudioPallet
}

impl TrainBundle {

	pub fn new(
		asset_server: &Res<AssetServer>,
		train_file_path : &str,
		translation : Vec3,
		speed : f32
	) -> Self {

		let train_type = TrainType::load_from_json(
			train_file_path.to_string()
		);

		let track_audio_path = train_type.track_audio_path;

		Self { 
			train: Train::new(asset_server, train_file_path, translation), 
			locomotion: Locomotion::new(speed), 
			transform: Transform::from_translation(translation), 
			visibility: Visibility::default(), 
			audio:ContinuousAudioPallet::new(
				vec![(
					"tracks".to_string(),
					ContinuousAudio::new(
						asset_server, 
						track_audio_path, 
						0.1,
						false
					)
				)]
			)
		}
	}
}

impl Train {
	pub fn new (
			asset_server: &Res<AssetServer>,
			train_file_path : &str,
			translation : Vec3
		) -> Train {

		let train_type = TrainType::load_from_json(
			train_file_path.to_string()
		);
		
		let carriage_text_vector: Vec<String> = train_type.carriages;
		let smoke_frames: Vec<String> = train_type.smoke.unwrap_or(vec![]);
		let horn_audio: Option<TransientAudio> = train_type.horn_audio_path.map(
			|path| {
				TransientAudio::new(
					path, 
					asset_server, 
					2.0, 
					false, 
					1.0
				)
		});
		
		let mut carriages : Vec<TrainCarriageBundle> = vec![];
		let smoke = TrainSmokeBundle::new(
			smoke_frames,
			translation
		);

		let mut carriage_translation = translation.clone();
		for carriage_text in carriage_text_vector {
			carriages.push(
				TrainCarriageBundle::new(
					carriage_text,
					carriage_translation
				)
			);
			carriage_translation.x -= 85.0;
		}
		
		Train {
			carriages,
			smoke,
			horn_audio
		}
	}

	pub fn update_speed(
		mut locomotion_query : Query<&mut Locomotion, With<Train>>,
		new_speed :f32
	) {
		for mut locomotion in locomotion_query.iter_mut() {
			locomotion.speed = new_speed;
		}
	}
}

impl Component for Train {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(
            |mut world, entity, _component_id| {
				// Step 1: Extract components from the pallet
				let train: Option<Train> = {
                    let entity_mut = world.entity(entity);
                    entity_mut.get::<Train>()
                        .map(|train: &Train| train.clone())
                };

				// Step 2: Spawn child entities and collect their IDs
                let mut commands = world.commands();
                let mut carriage_entities: Vec<Entity> = vec![];
                
                if let Some(train) = train {
                    commands.entity(entity).with_children(
						|parent| {
                        for carriage in train.carriages {
							carriage_entities.push(
								parent.spawn(carriage).id()
							);
						}
						parent.spawn(train.smoke);
                    });

					if let Some(horn_audio) = train.horn_audio{
						let mut engine = commands.entity(
							carriage_entities[0]
						);
				
						engine.insert((
							Clickable::new(
								vec![
									InputAction::PlaySound(
										"horn".to_string()
									)
								],
								Vec2::new(110.0, 60.0),
							), 
							TransientAudioPallet::new(
								vec![(
									"horn".to_string(),
									horn_audio,
								)]
							))
						);
					};
                }
            }
        );
    }
}