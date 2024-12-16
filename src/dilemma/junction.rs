use bevy::{
	prelude::*, 
	sprite::Anchor, 
	text::LineBreak
};

use crate::{
	lever::{
		OPTION_1_COLOR, 
		OPTION_2_COLOR
	}, 
	train::{
        TrainBundle,
        STEAM_TRAIN
    },
	track::TrackBundle,
	person::{
		PERSON,
		PersonSprite,
		BounceAnimation,
		EmoticonSprite,
	},
	motion::PointToPointTranslation
};

#[derive(Resource)]
pub struct TrainJunction{
	train : Entity,
	track : Vec<Entity>
}

impl TrainJunction{

	pub fn spawn(
			commands : &mut Commands,
			asset_server: &Res<AssetServer>,
			dilemma: &Dilemma
		) {

		let final_position = Vec3::new(
			90.0 * dilemma.countdown_duration_seconds,
			0.0, 
			0.0
		);

		let train: Entity = commands.spawn(
			TrainBundle::new(
				asset_server,
				STEAM_TRAIN,
				Vec3::new(50.0, -5.0, 1.0),
				0.0
			)
		).id();

		let color = match dilemma.default_option {
			None => Color::WHITE,
			Some(ref option) if *option == 1 => OPTION_1_COLOR,
			Some(_) =>  OPTION_2_COLOR,
		};

		let lower_track_y: f32 = -40.0;
		let upper_track_y: f32 = 60.0;

		let main_track_translation_end: Vec3 = Vec3::new(-1700.0, lower_track_y, 0.0);
		let main_track_translation_start: Vec3 = main_track_translation_end + final_position;
		let main_track: TrackBundle = TrackBundle::new(
			600, 
			main_track_translation_start
		);

		let track_1_translation_end: Vec3 = Vec3{x : 1000.0 , y : lower_track_y, z: 0.0};
		let track_1_translation_start: Vec3= track_1_translation_end + final_position;
		let track_1: TrackBundle = TrackBundle::new(
			300, 
			track_1_translation_start
		);

		let track_2_translation_end: Vec3 = Vec3{x : 1000.0 , y : upper_track_y, z: 0.0};
		let track_2_translation_start: Vec3 = track_2_translation_end + final_position;
		let track_2: TrackBundle = TrackBundle::new(
			300, 
			track_2_translation_start
		);
	
		let main_track : Entity = commands.spawn(main_track).id();
		let track_1 : Entity = commands.spawn(track_1).id();
		let track_2: Entity = commands.spawn(track_2).id();

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
						(
							Text2d::new(person.clone()),
							TextFont{
								font_size : 12.0,
								..default()
							},
							TextLayout{
								justify : JustifyText::Left, 
								linebreak: LineBreak::WordBoundary
							},
							Transform::from_translation(
								position
							),
							Anchor::BottomCenter,
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
						(
							Text2d::new(person.clone()),
							TextFont{
								font_size : 12.0,
								..default()
							},
							TextLayout{
								justify : JustifyText::Left, 
								linebreak: LineBreak::WordBoundary
							},
							Transform::from_translation(
								position
							),
							Anchor::BottomCenter,
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
		for track in self.track.clone() {
			commands.entity(track).despawn();
		}
	}
}