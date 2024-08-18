
use bevy::prelude::*;
use crate::text::TextSpriteBundle;

#[derive(Component)]
pub struct Track;

#[derive(Bundle)]
pub struct TrackBundle{
	marker : Track,
	text : TextSpriteBundle
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
			text : TextSpriteBundle::new(
				Self::generate_track(length),
				translation
			)
		}
	}
}