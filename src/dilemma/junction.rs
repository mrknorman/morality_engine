use bevy::{prelude::*, sprite::Anchor, text::BreakLineOn};
use crate::dilemma::Dilemma;
use crate::{
    train::{
		Train, 
		TrainEntities, 
		Track,
		STEAM_TRAIN
	},
    lever::{
		OPTION_1_COLOR, 
		OPTION_2_COLOR,
	},
    person::{
		PERSON,
		PersonSprite,
		BounceAnimation,
		EmoticonSprite
	},
	motion::PointToPointTranslation
};

#[derive(Component)]
pub struct LeverTrackTransform{
	pub index : usize,
	pub initial : Vec3,
	pub left : Vec3,
	pub right : Vec3,
	pub random : Vec3
}

#[derive(Resource)]
pub struct TrainJunction{
	train : TrainEntities,
	track : Vec<Entity>
}

impl TrainJunction{

	pub fn spawn(
			commands : &mut Commands,
			dilemma: &Dilemma
		) {

		let final_position = Vec3::new(
			90.0 * dilemma.countdown_duration_seconds,
			0.0, 
			0.0
		);

		let train: TrainEntities = Train::new(
			Some(STEAM_TRAIN.engine.to_string()),
			STEAM_TRAIN.carriages.iter().map(|&s| s.to_string()).collect(),
			STEAM_TRAIN.smoke.as_ref().map(|sm| sm.iter().map(|&s| s.to_string()).collect()).unwrap(),
			Vec3::new(100.0, -75.0, 1.0),
			0.0
		).spawn(commands);

		let color = match dilemma.default_option {
			None => Color::WHITE,
			Some(ref option) if *option == 1 => OPTION_1_COLOR,
			Some(_) =>  OPTION_2_COLOR,
		};

		let lower_track_y: f32 = -40.0;
		let upper_track_y: f32 = 60.0;

		let main_track_translation_end: Vec3 = Vec3::new(-1700.0, lower_track_y, 0.0);
		let main_track_translation_start: Vec3 = main_track_translation_end + final_position;
		let main_track: Track = Track::new(
			600, 
			color,
			main_track_translation_start
		);

		let track_1_translation_end: Vec3 = Vec3{x : 1000.0 , y : lower_track_y, z: 0.0};
		let track_1_translation_start: Vec3= track_1_translation_end + final_position;
		let track_1: Track = Track::new(
			300, 
			OPTION_1_COLOR,
			track_1_translation_start
		);

		let track_2_translation_end: Vec3 = Vec3{x : 1000.0 , y : upper_track_y, z: 0.0};
		let track_2_translation_start: Vec3 = track_2_translation_end + final_position;
		let track_2: Track = Track::new(
			300, 
		    OPTION_2_COLOR,
			track_2_translation_start
		);
	
		let main_track : Entity = main_track.spawn(commands);
		let track_1 : Entity = track_1.spawn(commands);
		let track_2: Entity = track_2.spawn(commands);

		commands.entity(main_track).insert(
			PointToPointTranslation::new(
				main_track_translation_start, 
				main_track_translation_end,
				dilemma.countdown_duration_seconds
			)
		);
		commands.entity(track_1).insert((
			PointToPointTranslation::new(
				track_1_translation_start, 
				track_1_translation_end,
				 dilemma.countdown_duration_seconds
			),
			LeverTrackTransform{
				index : 1, 
				initial : track_1_translation_end,
				left : Vec3{x: 0.0, y: 0.0, z: 0.0},
				right : Vec3{x: 0.0, y: -100.0, z: 0.0},
				random : Vec3{x: 0.0, y: -50.0, z: 0.0}
			})
		);
		commands.entity(track_2).insert((
			PointToPointTranslation::new(
				track_2_translation_start, 
				track_2_translation_end,
				 dilemma.countdown_duration_seconds
			),
			LeverTrackTransform{
				index : 2,
				initial : track_2_translation_end,
				left : Vec3{x: 0.0, y: 0.0, z: 0.0},
				right : Vec3{x: 0.0, y: -100.0, z: 0.0},
				random : Vec3{x: 0.0, y: -50.0, z: 0.0}
			}
		));

	
		let person = String::from(PERSON);
		for _ in 0..dilemma.options[0].consequences.total_fatalities {
			let position: Vec3 = Vec3::new(-800.0, 0.0, 0.0);
			commands.entity(track_1).with_children(|parent| {
					parent.spawn(
						(Text2dBundle {
							text : Text {
								sections : vec![
									TextSection::new(
										person.clone(),
										TextStyle {
											font_size: 12.0,
											..default()
									})
								],
								justify : JustifyText::Left, 
								linebreak_behavior: BreakLineOn::WordBoundary
							},
							transform: Transform::from_translation(
								position
							),
							text_anchor : Anchor::BottomCenter,
							..default()
						},
						PersonSprite::new(),
						BounceAnimation::new(40.0, 60.0)
						)
					).with_children(
						|parent| {
							EmoticonSprite::new().spawn_with_parent(parent);
						}
					);	
				}
			);
		}
	
		for _ in 0..dilemma.options[1].consequences.total_fatalities {
			let position: Vec3 = Vec3::new(-800.0, 0.0, 0.0);
			commands.entity(track_2).with_children(|parent| {
					parent.spawn(
						(Text2dBundle {
							text : Text {
								sections : vec![
									TextSection::new(
										person.clone(),
										TextStyle {
											font_size: 12.0,
											..default()
									})
								],
								justify : JustifyText::Left, 
								linebreak_behavior: BreakLineOn::WordBoundary
							},
							transform: Transform::from_translation(
								position
							),
							text_anchor : Anchor::BottomCenter,
							..default()
						},
						PersonSprite::new(),
						BounceAnimation::new(40.0, 60.0)
						)
					).with_children(
						|parent| {
							EmoticonSprite::new().spawn_with_parent(parent);
						}
					);	
				}
			);
		}
		
		let track = vec![main_track, track_1, track_2];
		let junction: TrainJunction = TrainJunction{
			train,
			track
		};

		commands.insert_resource(junction);
	}

	pub fn despawn( 
		&self,
		commands: &mut Commands
	) {
		//self.train.despawn(commands);
		
		for track in self.track.clone() {
			commands.entity(track).despawn();
		}
	}
}