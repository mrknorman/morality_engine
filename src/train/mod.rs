use bevy::prelude::*;
use std::time::Duration;
use rand::Rng;

use crate::io_elements::{NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::audio::play_sound_once;

const SMOKE_TEXT_FRAMES: [&str; 13] = [
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

const BACK_CARRIDGE_TEXT : &str = "\n
      _____    
  __|[_]|__
 |[] [] []|
_|________|
  oo    oo \n";

const MIDDLE_CARRIDGE_TEXT : &str = "\n
             
 ___________ 
 [] [] [] [] 
_[_________]
'oo      oo '\n";

const COAL_TRUCK_TEXT : &str = "\n
                                                     
 _______  
 [_____(__  
_[_________]_
  oo    oo ' \n";

const TRAIN_ENGINE_TEXT : &str = "\n
                   
____      
 ][]]_n_n__][.
_|__|________)<
oo 0000---oo\\_\n";

pub struct TrainText{
	pub train_track_text : Option<String>,
	pub train_engine_text : Option<String>,
	pub carridge_text_vector : Vec<String>,
	pub smoke_text_frames : Vec<String>
}

impl TrainText {

	pub fn new(
		button_engine : bool,
		track_length : usize
	) -> TrainText {

		let train_track_text : Option<String> = if track_length > 0 {
			Some(TrainTrack::generate_train_track_text(track_length))} else {None};
		
		if button_engine {
			TrainText{
				train_track_text,
				train_engine_text : Some(String::from(TRAIN_ENGINE_TEXT)),
				carridge_text_vector : vec![
					String::from(COAL_TRUCK_TEXT),
					String::from(MIDDLE_CARRIDGE_TEXT),
					String::from(BACK_CARRIDGE_TEXT)
				],
				smoke_text_frames : SMOKE_TEXT_FRAMES.iter().map(
					|&s| String::from(s)
				).collect()
			}
		} else {
			TrainText{
				train_track_text,
				train_engine_text : None,
				carridge_text_vector : vec![
					String::from(TRAIN_ENGINE_TEXT),
					String::from(COAL_TRUCK_TEXT),
					String::from(MIDDLE_CARRIDGE_TEXT),
					String::from(BACK_CARRIDGE_TEXT)
				],
				smoke_text_frames : SMOKE_TEXT_FRAMES.iter().map(
					|&s| String::from(s)
				).collect()
			}
		}
	}
}

#[derive(Component)]
pub struct TrainWhistle;

#[derive(Component)]
pub struct TrainTrack {
	pub text: String,
	pub color : Color,
	pub translation : Vec3
}

impl TrainTrack {

	pub fn new(
		text: String, 
		color : Color,
		translation : Vec3
	) -> TrainTrack {

		TrainTrack {
			text,
			color,
			translation
		}
	}

	pub fn generate_train_track_text(
		length: usize
	) -> String {
		let tilde_section = "~".repeat(length);
		let train_track_text = format!("\n{}\n", tilde_section);
		train_track_text
	}

	pub fn new_from_length(
			length: usize, 
			color : Color, 
			translation : Vec3
		) -> TrainTrack {
		
		TrainTrack {
			text: TrainTrack::generate_train_track_text(length),
			color,
			translation
		}
	}

	pub fn spawn_with_parent(self, parent: &mut ChildBuilder<'_>) -> Entity  {

		let text = self.text.clone();

		let translation = self.translation;

		parent.spawn((
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
				transform: Transform::from_xyz(
					translation.x,
					translation.y, 
					translation.z
				),
				..default()
			})).id()
	}

	pub fn spawn(self, commands : &mut Commands) -> Entity  {

		let text: String = self.text.clone();
		let color: Color = self.color;

		let translation = self.translation;

		commands.spawn((
			self,
			Text2dBundle {
				text : Text {
					sections : vec!(
						TextSection::new(text, TextStyle {
							font_size : 12.0,
							color,
							..default()
						})
					),
					justify : JustifyText::Center, 
					..default()
				},
				transform: Transform::from_xyz(
					translation.x,
					translation.y, 
					translation.z
				),
				..default()
			})).id()
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
	pub track : Option<TrainTrack>,
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
			train_track_text : Option<String>,
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
		
		// Move track below train:
		let track_displacement: Vec3 = Vec3::new(110.0, -40.0, 0.0);
		let track : Option<TrainTrack> = train_track_text.map(|text| TrainTrack::new(
			text, 
			Color::WHITE,
			translation + track_displacement
		));

		Train {
			track,
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