use bevy::prelude::*;
use std::time::Duration;
use rand::Rng;

use crate::io_elements::{NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::audio::play_sound_once;

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

static STEAM_TRAIN_ENGINE: &str = "\n
                   
____      
 ][]]_n_n__][.
_|__|________)<
oo 0000---oo\\_\n";

pub struct TrainSprite<'a> {
    pub engine: &'a str,
    pub carriages: &'a [&'a str],
    pub smoke: Option<&'a [&'a str]>,
}
pub static STEAM_TRAIN: TrainSprite = TrainSprite {
    engine: STEAM_TRAIN_ENGINE,
    carriages: &[COAL_TRUCK, MIDDLE_CARRIAGE, BACK_CARRIAGE],
    smoke: Some(&SMOKE_TEXT_FRAMES),
};


#[derive(Component)]
pub struct TrainWhistle;

#[derive(Component)]
pub struct Track {
	pub text: String,
	pub color : Color,
	pub translation : Vec3
}

impl Track {

	pub fn generate_track(
		length: usize
	) -> String {
		"~".repeat(length)
	}
	
	pub fn new(
			length: usize, 
			color : Color, 
			translation : Vec3
		) -> Track {
		
		Track {
			text: Track::generate_track(length),
			color,
			translation
		}
	}
	pub fn bundle(self) -> impl Bundle {

		let text: String = self.text.clone();
		let translation: Vec3 = self.translation.clone();
		(
			self,
			Text2dBundle {
				text : Text {
					sections : vec!(
						TextSection::new(text, TextStyle {
							font_size : 12.0,
							..default()
						})
					),
					justify : JustifyText::Center, 
					..default()
				},
				transform: Transform::from_translation(translation),
				..default()
			}
		)
	}
	pub fn spawn_with_parent(self, parent: &mut ChildBuilder<'_>) -> Entity{
		parent.spawn(self.bundle()).id()
	}
	pub fn spawn(self, commands : &mut Commands) -> Entity  {
		commands.spawn(self.bundle()).id()
	}
}

#[derive(Component)]
pub struct TrainSmoke {
	pub frame_index : usize,
	pub text_frames : Vec<String>,
	pub translation : Vec3,
	pub timer: Timer,
	pub speed : f32
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
			),
			speed
		}
	}

	pub fn spawn(self, parent: &mut ChildBuilder<'_>) -> Entity {

		parent.spawn((
			Text2dBundle {
				text : Text {
					sections : vec!(
						TextSection::new(self.text_frames[0].clone(), TextStyle {
							font_size : 12.0,
							..default()
						})
					),
					justify : JustifyText::Center, 
					..default()
				},
				transform: Transform::from_xyz(
					self.translation.x,
					self.translation.y,
					self.translation.z
				),
				..default()
			}, self)
		).id()
	}
}
#[derive(Component)]
pub struct TrainPart{
	pub text : String,
	pub translation : Vec3,
	pub timer: Timer,
	pub speed : f32
}

impl TrainPart {

	pub fn new(
			text : String,
			translation : Vec3,
			speed : f32
		) -> TrainPart {

		TrainPart{
			text,
			translation,
			timer : Timer::new(
				Duration::from_millis(100), 
				TimerMode::Repeating
			),
			speed
		}
	}
	
	pub fn spawn(
			self,
			parent: &mut ChildBuilder<'_>
		) -> Entity {
		
		parent.spawn((
			Text2dBundle {
				text : Text {
					sections : vec!(
						TextSection::new(self.text.clone(), TextStyle {
							font_size : 12.0,
							..default()
						})
					),
					justify : JustifyText::Center, 
					..default()
				},
				transform: Transform::from_xyz(
					self.translation.x, 
					self.translation.y, 
					self.translation.z
				),
				..default()
			},
			self)
		).id()
	}

}

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

#[derive(Component)]
pub struct Train{
	pub track : Option<Track>,
	pub engine : Option<TrainEngine>,
	pub carridges : Vec<TrainPart>,
	pub smoke : TrainSmoke
}
pub struct TrainEntities{
	pub engine : Option<Entity>,
	pub train : Entity
}

impl TrainEntities {

	pub fn despawn(
		&self, 			
		commands: &mut Commands
	) {
		if self.engine.is_some() {
			commands.entity(self.engine.unwrap()).despawn_recursive();
		}

		commands.entity(self.train).despawn_recursive();
	}
}

#[derive(Component)]
pub struct TrainNode;

impl Train {
	pub fn new (
			train_engine_text : Option<String>,
			carridge_text_vector : Vec<String>,
			smoke_text_frames : Vec<String>,
			mut translation : Vec3,
			speed : f32
		) -> Train {
		
		let mut carridges : Vec<TrainPart> = vec![];
		let smoke = TrainSmoke::new(
			smoke_text_frames,
			translation,
			speed
		);

		if train_engine_text.is_some() {
			translation.x -= 70.0;
		} else {
			translation.x += 10.0;
		}

		for carridge_text in carridge_text_vector {
			carridges.push(
				TrainPart::new(
					carridge_text,
					translation,
					speed
				)
			);
			translation.x -= 70.0;
		}
		translation.x += 70.0;

		let engine : Option<TrainEngine> = train_engine_text.map(|text| TrainEngine::new(
			text, 
			Vec3::new(0.0, 0.0, 1.0),
			speed
		));

		Train {
			track: None,
			engine,
			carridges,
			smoke
		}	
	}

	pub fn spawn(
			self,
			commands: &mut Commands
		) -> TrainEntities {
		
		let engine_entity = if self.engine.is_some() {
			Some(self.engine.unwrap().spawn(commands))
		} else {None};

		let train_entity : Entity = commands.spawn((TrainNode, Text2dBundle {
			text : Text {
				sections : vec!(
					TextSection::new("", TextStyle {
						font_size : 12.0,
						..default()
					})
				),
				justify : JustifyText::Center, 
				..default()
			},
			transform: Transform::from_xyz(-0.0, 70.0, 1.0),
			..default()
		})).with_children(
			|parent : &mut ChildBuilder<'_>| {
				for part in self.carridges {
					part.spawn(parent);
				}
				self.smoke.spawn(parent);

				if self.track.is_some() {
					self.track.unwrap().spawn_with_parent(parent);
				}
			}).id();

		TrainEntities{
			engine : engine_entity,
			train : train_entity,
		}
	}
	
	pub fn wobble(
			time: Res<Time>, // Inject the Time resource to access the game time
			mut transform_query: Query<(&mut Transform, &mut TrainPart) >,
			mut button_query: Query<(&mut Style, &mut TrainEngine)>,
			mut smoke_query: Query<(&mut Text, &mut TrainSmoke)>
		) {

		let mut rng = rand::thread_rng(); // Random number generator

		for (mut style, mut train_part) in button_query.iter_mut() {
			if train_part.timer.tick(time.delta()).finished() {

				// Calculate offset using sine and cosine functions for smooth oscillation
				let dx = rng.gen_range(-1.0..=1.0);
				let dy = rng.gen_range(-1.0..=1.0);  
	
				// Apply the calculated offsets to the child's position
				style.top = Val::Px(train_part.translation.x + dx as f32);
				style.left = Val::Px(train_part.translation.y + dy as f32);
			}
		}

		let _time_seconds = time.elapsed_seconds_f64() as f32; // Current time in seconds

		for (mut transform, mut train_part) in transform_query.iter_mut() {
			if train_part.timer.tick(time.delta()).finished() {
				// Calculate offset using sine and cosine functions for smooth oscillation
				let dx = rng.gen_range(-1.0..=1.0);
				let dy = rng.gen_range(-1.0..=1.0);  

				// Apply the calculated offsets to the child's position
				transform.translation.x = train_part.translation.x + dx as f32;
				transform.translation.y = train_part.translation.y + dy as f32;
			}
		}

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

	pub fn move_train(
		time: Res<Time>, // Inject the Time resource to access the game time
		mut transform_query: Query<(&mut Transform, &mut TrainPart), Without<TrainSmoke>>,
		mut button_query: Query<(&mut Style, &mut TrainEngine)>,
		mut smoke_query: Query<(&mut Transform, &mut TrainSmoke)>
	) {

		let time_seconds: f32 = time.delta().as_millis() as f32 / 1000.0; // Current time in seconds
		
		for (mut style, mut train_part) in button_query.iter_mut() {
			// Apply the calculated offsets to the child's position

			let dx = train_part.speed*time_seconds;
			train_part.translation.x += dx;
			style.left = Val::Px(train_part.translation.x);
		}

		for (mut transform, mut train_part) in transform_query.iter_mut() {
			// Apply the calculated offsets to the child's position
			let dx = train_part.speed*time_seconds;
			train_part.translation.x += dx;
			transform.translation.x = train_part.translation.x + dx;
		}

		for (mut transformm, mut smoke_part) in smoke_query.iter_mut() {
			let dx = smoke_part.speed*time_seconds;
			smoke_part.translation.x += dx;
			transformm.translation.x = smoke_part.translation.x + dx;
		}
	}

	pub fn update_speed(
		mut transform_query: Query<&mut TrainPart, Without<TrainSmoke>>,
		mut button_query: Query<&mut TrainEngine>,
		mut smoke_query: Query<&mut TrainSmoke>,
		new_speed :f32
	) {

		for mut train_part in button_query.iter_mut() {
			// Apply the calculated offsets to the child's position
			train_part.speed = new_speed;
		}

		for mut train_part in transform_query.iter_mut() {
			train_part.speed = new_speed;
		}

		for mut smoke_part in smoke_query.iter_mut() {
			smoke_part.speed = new_speed;
		}
	}

}