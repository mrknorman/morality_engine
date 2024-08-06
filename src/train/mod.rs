use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

use crate::{
    audio::play_sound_once,
    io_elements::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    text::{TextComponent, TextSprite},
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

static BACK_CARRIAGE: &str = "\n
      _____    
  __|[_]|__
 |[] [] []|
_|________|
  oo    oo \n";

static MIDDLE_CARRIAGE: &str = "\n
             
 ___________ 
 [] [] [] [] 
_[_________]
'oo      oo '\n";

static COAL_TRUCK: &str = "\n
                                                     
 _______  
 [_____(__  
_[_________]_
  oo    oo ' \n";

static ENGINE: &str = "\n
                   
____      
 ][]]_n_n__][.
_|__|________)<
oo 0000---oo\\_\n";

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

// Trein Component:

#[derive(Component)]
pub struct TrainComponent;

#[derive(Component)]
pub struct TranslationAnchor{
	pub translation : Vec3
}

impl TranslationAnchor {

	pub fn new(translation : Vec3) -> TranslationAnchor {
		TranslationAnchor {
			translation
		}
	}
}

#[derive(Component, Clone)]
pub struct Wobble{
	pub timer: Timer
}

impl Wobble {
	pub fn new() -> Wobble {
		Wobble{
			timer : Timer::new(
				Duration::from_millis(100), 
				TimerMode::Repeating
			)
		}
	}

	pub fn wobble(
		time: Res<Time>, // Inject the Time resource to access the game time
		mut wobble_query: Query<(&mut Transform, &mut Wobble, &TranslationAnchor)>
	) {

		let mut rng = rand::thread_rng(); // Random number generator

		for (mut transform, mut wobble, translation_anchor) in wobble_query.iter_mut() {
			if wobble.timer.tick(time.delta()).finished() {
				// Calculate offset using sine and cosine functions for smooth oscillation
				let dx = rng.gen_range(-1.0..=1.0);
				let dy = rng.gen_range(-1.0..=1.0);  

				// Apply the calculated offsets to the child's position
				transform.translation.x = translation_anchor.translation.x + dx as f32;
				transform.translation.y = translation_anchor.translation.y + dy as f32;
			}
		}
	}
}

#[derive(Component, Clone)]
pub struct Locamotion{
	pub speed: f32
} 

impl Locamotion{

	pub fn new(speed: f32) -> Locamotion  {
		Locamotion{
			speed
		}
	}

	pub fn locamote(
		time: Res<Time>,
		mut locomotion_query: Query<(&Locamotion, &mut Transform)>
	) {
		let time_seconds: f32 = time.delta().as_secs_f32(); // Current time in seconds

		for (locomotion, mut transform) in locomotion_query.iter_mut() {
			let dx = locomotion.speed*time_seconds;
			transform.translation.x += dx;
		}
	}
}

#[derive(Component, Clone)]
pub struct TrainCarridge{
	pub text : String,
	pub translation : Vec3,
}

impl TrainCarridge {

	pub fn new(
			text : String,
			translation : Vec3,
			speed : f32
		) -> TrainCarridge {

		TrainCarridge{
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
	pub frame_index : usize,
	pub text_frames : Vec<String>,
	pub translation : Vec3,
	pub timer: Timer
}

impl TrainSmoke {

	pub fn new(
		smoke_text_frames : Vec<String>, 
		translation : Vec3,
		speed : f32
		) -> TrainSmoke{

		TrainSmoke{
			frame_index : 0,
			text_frames : smoke_text_frames,
			translation : Vec3::new(
				translation.x - 25.0,
				translation.y + 10.0,
				translation.z,
			),
			timer: Timer::new(
				Duration::from_millis(100), 
				TimerMode::Repeating
			)
		}
	}

	pub fn spawn(self, parent: &mut ChildBuilder<'_>) -> Entity {

		let translation: Vec3 = self.translation.clone();
		let text: String  = self.text_frames[0].clone();

		parent.spawn((
			self,
			TrainComponent,
			TextSprite::new(
				text,
				translation
			))
		).id()
	}
}

#[derive(Component)]
pub struct TrainWhistle;

#[derive(Component, Clone)]
pub struct Train{
	pub carridges : Vec<TrainCarridge>,
	pub smoke : TrainSmoke,
	pub translation : Vec3,
	pub speed : f32
}

#[derive(Component)]
pub struct TrainNode;

impl Train {
	pub fn new (
			train_sprite : TrainSprite,
			mut translation : Vec3,
			speed : f32
		) -> Train {
		
		let carridge_text_vector: Vec<String> =
			train_sprite.carriages.iter().map(|&s| s.to_string()).collect();
		let smoke_text_frames: Vec<String> = 
			train_sprite.smoke.as_ref().map(
				|sm| sm.iter().map(|&s| s.to_string()).collect()
			).unwrap();
		
		let mut carridges : Vec<TrainCarridge> = vec![];
		let smoke = TrainSmoke::new(
			smoke_text_frames,
			translation,
			speed
		);

		for carridge_text in carridge_text_vector {
			carridges.push(
				TrainCarridge::new(
					carridge_text,
					translation,
					speed
				)
			);
			translation.x -= 70.0;
		}
		translation.x += 70.0;

		/*
		let engine : Option<TrainEngine> = train_engine_text.map(|text| TrainEngine::new(
			text, 
			Vec3::new(0.0, 0.0, 1.0),
			speed
		));
		*/

		Train {
			carridges,
			smoke,
			translation,
			speed
		}
	}

	pub fn spawn(
			self,
			commands: &mut Commands
		) -> Entity {
		
		/* 
		let engine_entity = if self.engine.is_some() {
			Some(self.engine.unwrap().spawn(commands))
		} else {None};
		*/

		let translation: Vec3 = self.translation.clone();
		let speed = self.speed;
		let train_entity : Entity = commands.spawn((
			self.clone(),
			Locamotion::new(self.speed),
			TransformBundle::from_transform(
				Transform::from_translation(translation)
			),
			VisibilityBundle::default()
		)).with_children(
			|parent : &mut ChildBuilder<'_>| {
				for part in self.carridges {
					part.spawn(parent);
				}
				self.smoke.spawn(parent);
			}
		).id();

		train_entity
	}

	pub fn animate_smoke(
		time: Res<Time>, // Inject the Time resource to access the game time
		mut smoke_query: Query<(&mut Text, &mut TrainSmoke)>
	) {
		for (mut text, mut smoke_part) in smoke_query.iter_mut() {

			if smoke_part.timer.tick(time.delta()).finished() {
				smoke_part.frame_index += 1;

				text.sections[0].value.clone_from(
					&smoke_part.text_frames[smoke_part.frame_index % smoke_part.text_frames.len()]
				);
			}
		}
	}
	
	pub fn whistle(
		mut interaction_query: Query<
			(&Children, &Interaction, &TrainWhistle),
			(Changed<Interaction>, With<Button>, With<TrainWhistle>),
		>,
		mut text_query: Query<&mut Text>,
		asset_server : Res<AssetServer>,
		mut commands: Commands
		) {
			
		for (children, interaction, _) in &mut interaction_query {
	
			let text_entity = children.iter().next();
	
			if let Some(text_entity) = text_entity {
				if let Ok(mut text) = text_query.get_mut(*text_entity) {
					match *interaction {
						Interaction::Pressed => {
							text.sections[0].style.color = PRESSED_BUTTON;
	
							play_sound_once(
								"sounds/horn.ogg", 
								&mut commands, 
								&asset_server
							);					
							
						}
						Interaction::Hovered => {
							text.sections[0].style.color = HOVERED_BUTTON;
						}
						Interaction::None => {
							text.sections[0].style.color = NORMAL_BUTTON;
						}
					}
				}
			}
		}
	}
	
	pub fn update_speed(
		mut locamotion_query : Query<&mut Locamotion, With<Train>>,
		new_speed :f32
	) {
		for mut locamotion in locamotion_query.iter_mut() {
			locamotion.speed = new_speed;
		}
	}
}


/* 
#[derive(Component)]
pub struct TrainEngine{
	pub text : String,
	pub translation : Vec3,
	pub timer: Timer,
	pub speed : f32
}

impl TrainEngine {

	pub fn new(
		text : String,
		translation : Vec3,
		speed : f32
	) -> TrainEngine {
		TrainEngine {
			text,
			translation,
			timer :  Timer::new(
				Duration::from_millis(100), 
				TimerMode::Repeating
			),
			speed
		}
	}

	pub fn spawn(
			self,
			commands: &mut Commands,
		) -> Entity {

		let text = self.text.clone();

		commands
			.spawn((NodeBundle {
				style: Style {
					// center button
					width: Val::Percent(100.),
					height: Val::Percent(100.),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					margin: UiRect {
						top : Val::Px(-71.0),
						left : Val::Px(80.0),
						..default()
					},
					..default()
				},
				..default()
			}, self))
			.with_children(|parent| {
				parent
					.spawn((ButtonBundle {
						style: Style {
							width: Val::Px(100.),
							height: Val::Px(45.),
							// horizontally center child text
							justify_content: JustifyContent::Center,
							// vertically center child text
							align_items: AlignItems::Center,
							..default()
						},
						background_color : bevy::prelude::BackgroundColor(
							Color::srgb(0.0, 0.0, 0.0)
						),
						..default()
					}, TrainWhistle
					
					))
					.with_children(|parent| {
						parent.spawn(TextBundle {
							text : Text {
								sections : vec![
									TextSection::new(text, TextStyle {
										font_size: 12.0,
										color: Color::srgb(0.9, 0.9, 0.9),
										..default()
									})
								],
								..default()
							},
							..default()
						}	
					);
					});
			})
			.id()
	}

}
*/