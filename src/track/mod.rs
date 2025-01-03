
use bevy::prelude::*;
use crate::text::TextSprite;

#[derive(Component)]
#[require(TextSprite, Text2d)]
pub struct Track;

impl Track{
	pub fn generate_track(
		length: usize
	) -> String {
		"~".repeat(length)
	}
	
	pub fn new(length: usize) -> (Track, Text2d) {
		(
			Track,
			Text2d::new(Self::generate_track(length))
		)
	}
}