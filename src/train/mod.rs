use std::{
	fs::File,
	io::BufReader
};

use bevy::{
	prelude::*,
	ecs::component::StorageType,
	audio::Volume
};
use serde::Deserialize;

use crate::{
    audio::{
		AudioPlugin, 
		ContinuousAudioPallet, 
		continuous_audio, 
		TransientAudio, 
		TransientAudioPallet
	}, 
	interaction::{
		Clickable, InputAction, InteractionPlugin
	}, motion::{
		Locomotion, 
		TranslationAnchor, 
		Wobble
	}, 
	text::{
		Animated, 
		TextSprite,
		TextFrames
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
				Animated::animate_text,
                Wobble::wobble,
				Locomotion::locomote,
            )
            .run_if(in_state(TrainSystemsActive::True))
        );

		app.register_required_components::<Train, Transform>();
        app.register_required_components::<Train, Visibility>();

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

#[derive(Component)]
#[require(Wobble, TextSprite, Text2d, TranslationAnchor)]
pub struct TrainCarriage;

impl TrainCarriage{
	pub fn new(
		text : String,
		translation : Vec3
	) -> (TrainCarriage, TranslationAnchor, Text2d, Transform) {

		(
			TrainCarriage,
			TranslationAnchor::new(
				translation
			),
			Text2d::new(text),
			Transform::from_translation(translation)
		)
	}
}

#[derive(Component)]
#[require(Animated)]
pub struct TrainSmoke;

#[derive(Clone)]
pub struct Train {
	pub carriages : Vec<String>,
	pub smoke_frames : Vec<String>,
	pub horn_audio : Option<TransientAudio>
}

impl Train {

	pub fn init(
		asset_server: &Res<AssetServer>,
		train_file_path : &str,
		translation : Vec3,
		speed : f32
	) -> (Train, Locomotion, Transform, ContinuousAudioPallet, ) {

		let train_type = TrainType::load_from_json(
			train_file_path.to_string()
		);

		(
			Train::new(asset_server, train_file_path), 
			Locomotion::new(speed), 
			Transform::from_translation(translation), 
			ContinuousAudioPallet::new(
				vec![(
					"tracks".to_string(),
					AudioPlayer::<AudioSource>(asset_server.load(
						train_type.track_audio_path
					)),
					PlaybackSettings{
						volume : Volume::new(0.1),
						..continuous_audio()
					}
				)]
			)
		)
	}

	pub fn new (
			asset_server: &Res<AssetServer>,
			train_file_path : &str
		) -> Train {

		let train_type = TrainType::load_from_json(
			train_file_path.to_string()
		);
		
		let smoke_frames: Vec<String> = train_type.smoke.unwrap_or(vec![]);
		let horn_audio: Option<TransientAudio> = train_type.horn_audio_path.map(
			|path| {
				TransientAudio::new(
					asset_server.load(path), 
					2.0, 
					false, 
					1.0
				)
		});

		Train {
			carriages : train_type.carriages,
			smoke_frames,
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

						let mut carriage_translation = Vec3::default();
                        for carriage in train.carriages {
							carriage_entities.push(
								parent.spawn(				
								TrainCarriage::new(
									carriage,
									carriage_translation
								)).id()
							);
							carriage_translation.x -= 85.0;
						}
						parent.spawn((
							TrainSmoke,
							Text2d(train.smoke_frames[0].clone()),
							TextFrames::new(train.smoke_frames),
							Transform::from_xyz(
								-35.0,
								32.0,
								1.0
							)
						));
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
									vec![horn_audio],
								)]
							))
						);
					};
                }
            }
        );
    }
}