
use bevy::prelude::*;
use crate::text::{TextComponent, TextSprite};

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
			TextSprite::new(
				text,
				translation
			)
		)
	}
	pub fn spawn_with_parent(self, parent: &mut ChildBuilder<'_>) -> Entity {
		parent.spawn(self.bundle()).id()
	}
	pub fn spawn(self, commands : &mut Commands) -> Entity  {
		commands.spawn(self.bundle()).id()
	}
}