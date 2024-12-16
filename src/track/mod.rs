
use bevy::prelude::*;
use crate::text::TextSprite;

#[derive(Component)]
pub struct Track;

#[derive(Bundle)]
pub struct TrackBundle{
	marker : Track,
	text_sprite : TextSprite,
	text : Text2d,
	transform : Transform
}

impl TrackBundle {
	pub fn generate_track(
		length: usize
	) -> String {
		"~".repeat(length)
	}
	
	pub fn new(
			length: usize, 
			translation : Vec3
		) -> Self {
		
		Self {
			marker : Track,
			text_sprite : TextSprite,
			text : Text2d::new(Self::generate_track(length)),
			transform : Transform::from_translation(translation)
		}
	}
}