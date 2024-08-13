use bevy::{
	prelude::*,
	ecs::system::EntityCommands 
};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use crate::{
    audio::{
		ContinuousAudio, 
		ContinuousAudioPallet,
		TransientAudio,
		TransientAudioPallet
	}, 
	interaction::{
		InteractionPlugin, 
		ClickAction, 
		Clickable
	}, motion::{
		Locomotion, 
		TranslationAnchor, 
		Wobble
	}, 
	text::{
		AnimatedTextSprite, 
		TextComponent, 
		TextSprite
	}
};

pub static STEAM_TRAIN: &str = "./steam_train.json";

pub struct TrainPlugin<T: States + Clone + Eq + Default> {
    active_state: T,
}

impl<T: States + Clone + Eq + Default> TrainPlugin<T> {
    pub fn new(active_state: T) -> Self {
        Self { active_state }
    }
}

impl<T: States + Clone + Eq + Default + 'static> Plugin for TrainPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
				AnimatedTextSprite::animate_text_sprites,
                Wobble::wobble,
				Locomotion::locomote,
            )
            .run_if(in_state(self.active_state.clone()))
        ).add_plugins(InteractionPlugin::new(
			self.active_state.clone())
		);
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

// Train Component:
#[derive(Component)]
pub struct TrainComponent;

#[derive(Component, Clone)]
pub struct TrainCarriage{
	pub text : String,
	pub translation : Vec3
}

impl TrainCarriage {

	pub fn new(
			text : String,
			translation : Vec3
		) -> TrainCarriage {

		TrainCarriage{
			text,
			translation
		}
	}
	
	pub fn spawn(
			self,
			parent: &mut ChildBuilder<'_>
		) -> Entity {

		let translation: Vec3 = self.translation.clone();
		let text: String  = self.text.clone();
		
		parent.spawn((
			self,
			TrainComponent,
			TranslationAnchor::new(
				translation
			),
			Wobble::new(),
			TextSprite::new(
				text,
				translation
			))
		).id()
	}
}

#[derive(Component, Clone)]
pub struct TrainSmoke {
	pub frames : Vec<String>,
	pub translation : Vec3
}

impl TrainSmoke {

	pub fn new(
			smoke_frames : Vec<String>, 
			translation : Vec3
		) -> TrainSmoke{

		TrainSmoke{
			frames : smoke_frames,
			translation : Vec3::new(
				translation.x - 25.0,
				translation.y + 20.0,
				translation.z,
			)
		}
	}

	pub fn spawn(self, parent: &mut ChildBuilder<'_>) -> Entity {

		let translation: Vec3 = self.translation.clone();
		parent.spawn((
			self.clone(),
			TrainComponent,
			AnimatedTextSprite::from_vec(
				self.frames.iter().map(
					|s| s.to_string()
				).collect(),
				0.1,
				translation
			)
		)).id()
	}
}

#[derive(Component, Clone)]
pub struct Train{
	pub carriages : Vec<TrainCarriage>,
	pub smoke : TrainSmoke,
	pub translation : Vec3,
	pub track_audio_path : String,
	pub horn_audio_path : Option<String>,
	pub speed : f32
}

impl Train {
	pub fn new (
			train_file_path : &str,
			translation : Vec3,
			speed : f32
		) -> Train {

		let train_type = TrainType::load_from_json(
			train_file_path.to_string()
		);
		
		let carriage_text_vector: Vec<String> = train_type.carriages;
		let smoke_frames: Vec<String> = train_type.smoke.unwrap_or(vec![]);
		let track_audio_path = train_type.track_audio_path;
		let horn_audio_path = train_type.horn_audio_path;
		
		let mut carriages : Vec<TrainCarriage> = vec![];
		let smoke = TrainSmoke::new(
			smoke_frames,
			translation
		);

		let mut carriage_translation = translation.clone();
		for carriage_text in carriage_text_vector {
			carriages.push(
				TrainCarriage::new(
					carriage_text,
					carriage_translation
				)
			);
			carriage_translation.x -= 70.0;
		}
		
		Train {
			carriages,
			smoke,
			translation,
			track_audio_path,
			horn_audio_path,
			speed
		}
	}

	pub fn spawn(
		self,
		commands: &mut Commands,
		asset_server: &Res<AssetServer>,
		parent_entity: Option<Entity> // New parameter to specify an optional parent entity
	) -> Entity {
		let translation: Vec3 = self.translation.clone();
	
		let mut carriage_entities: Vec<Entity> = vec![];
		let train_entity = if let Some(parent) = parent_entity {
			// If a parent entity is provided, spawn the train as a child of the parent
			commands.entity(parent).with_children(|parent| {
				parent.spawn((
					self.clone(),
					Locomotion::new(self.speed),
					TransformBundle::from_transform(Transform::from_translation(translation)),
					VisibilityBundle::default(),
				))
				.with_children(|parent| {
					for part in &self.carriages {
						carriage_entities.push(part.clone().spawn(parent));
					}
					self.smoke.clone().spawn(parent);
				});
			}).id()
		} else {
			// Otherwise, spawn the train as a top-level entity
			commands.spawn((
				self.clone(),
				Locomotion::new(self.speed),
				TransformBundle::from_transform(Transform::from_translation(translation)),
				VisibilityBundle::default(),
			))
			.with_children(|parent| {
				for part in self.carriages {
					carriage_entities.push(part.spawn(parent));
				}
				self.smoke.spawn(parent);
			})
			.id()
		};
	
		// Add continuous audio:
		let mut train = commands.entity(train_entity);
		ContinuousAudioPallet::insert(
			vec![(
				"tracks".to_string(),
				ContinuousAudio::new(self.track_audio_path, asset_server, 0.1),
			)],
			&mut train,
		);
	
		// Add transient audio:
		if let Some(horn_path) = &self.horn_audio_path {
			let mut engine = commands.entity(carriage_entities[0]);
	
			engine.insert(Clickable::new(
				ClickAction::PlaySound("horn".to_string()),
				Vec2::new(110.0, 60.0),
			));
			TransientAudioPallet::insert(
				vec![(
					"horn".to_string(),
					TransientAudio::new(horn_path.clone(), asset_server, 2.0, 1.0),
				)],
				&mut engine,
			);
		};
	
		train_entity
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