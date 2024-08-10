use bevy::prelude::*;
use std::time::Duration;
use crate::{
    text::{TextComponent, TextSprite, AnimatedTextSprite},
	motion::{TranslationAnchor, Wobble, Locomotion},
	interaction::{Clickable, ClickAction, clickable_system}
};

// Strings:
static SMOKE_TEXT_FRAMES: [&str; 13] = [
    ". . . . . o o o o o\n                    o",
    ". . . . o o o o o o\n                    .",
    ". . . o o o o o o .\n                    .",
    ". . . o o o o o o .\n                    .",
    ". . o o o o o o . .\n                    .",
    ". o o o o o o . . .\n                    .",
    "o o o o o o . . . .\n                    .",
    "o o o o o . . . . .\n                    o",
    "o o o o . . . . . o\n                    o",
    "o o o . . . . . o o\n                    o",
    "o o . . . . . o o o\n                    o",
    "o . . . . . o o o o\n                    o",
    ". . . . . o o o o o\n                    o",
];

static BACK_CARRIAGE: &str = 
"      _____    
  __|[_]|__
 |[] [] []|
_|________|
  oo    oo \n\n";

static MIDDLE_CARRIAGE: &str = 
"     
 ___________ 
 [] [] [] [] 
_[_________]
'oo      oo '\n\n";

static COAL_TRUCK: &str = 
"                                                 
_______  
 [_____(__  
_[_________]_
  oo    oo ' \n\n";

static ENGINE: &str = 
"          
____      
   ][]]_n_n__][.
  _ |__|________)<
  oo 0000---oo\\_\n\n";

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
				clickable_system,
            )
            .run_if(in_state(self.active_state.clone()))
        );
    }
}

// Train Sprites:
#[derive(Clone)]
pub struct TrainSprite<'a> {
    pub carriages: &'a [&'a str],
    pub smoke: Option<&'a [&'a str]>,
}
pub static STEAM_TRAIN: TrainSprite = TrainSprite {
    carriages: &[ENGINE, COAL_TRUCK, MIDDLE_CARRIAGE, BACK_CARRIAGE],
    smoke: Some(&SMOKE_TEXT_FRAMES),
};

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
				self.frames.iter().map(|s| s.to_string()).collect(),
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
	pub speed : f32
}

impl Train {
	pub fn new (
			train_sprite : TrainSprite,
			mut translation : Vec3,
			speed : f32
		) -> Train {
		
		let carriage_text_vector: Vec<String> = train_sprite.carriages.iter().map(
				|&s| s.to_string()
			).collect();
		let smoke_frames: Vec<String> = train_sprite.smoke.as_ref().map(
				|sm| sm.iter().map(|&s| s.to_string()).collect()
			).unwrap();
		
		let mut carriages : Vec<TrainCarriage> = vec![];
		let smoke = TrainSmoke::new(
			smoke_frames,
			translation
		);

		for carriage_text in carriage_text_vector {
			carriages.push(
				TrainCarriage::new(
					carriage_text,
					translation
				)
			);
			translation.x -= 70.0;
		}
		translation.x += 70.0;

		Train {
			carriages,
			smoke,
			translation,
			speed
		}
	}

	pub fn spawn(
			self,
			commands: &mut Commands
		) -> Entity {

		let translation: Vec3 = self.translation.clone();

		let mut carriage_entities: Vec<Entity> = vec![];
		let train_entity : Entity = commands.spawn((
			self.clone(),
			Locomotion::new(self.speed),
			TransformBundle::from_transform(
				Transform::from_translation(translation)
			),
			VisibilityBundle::default()
		)).with_children(
			|parent : &mut ChildBuilder<'_>| {
				for part in self.carriages {
					carriage_entities.push(part.spawn(parent));
				}
				self.smoke.spawn(parent);
			}
		).id();

		// Insert clickable element onto train engine only:
		commands.entity(carriage_entities[0]).insert(
			Clickable::new(
				ClickAction::PlaySound("sounds/horn.ogg"),
				Vec2::new(110.0, 60.0)
			)
		);

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