
use bevy::prelude::*;
use crate::text::{
	TextComponent, 
	TextSprite
};

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

	pub fn spawn(
		self, commands: &mut Commands, parent_entity: Option<Entity>
	) -> Entity {
		
        if let Some(parent) = parent_entity {
            // Spawn the track as a child of the parent entity
            commands.entity(parent).with_children(|parent| {
                parent.spawn(self.bundle());
            }).id()
        } else {
            // Spawn the track as a top-level entity
            commands.spawn(self.bundle()).id()
        }
    }
}