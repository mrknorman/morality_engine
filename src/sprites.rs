use bevy::{
	prelude::*,
	sprite::Anchor
};

use crate::colors::PRIMARY_COLOR;

// Separate module for general sprite creation functionality
pub struct SpriteFactory;

impl SpriteFactory {
    pub fn create_sprite_bundle(size: Vec2, position: Vec3) -> SpriteBundle {
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(size),
                color : PRIMARY_COLOR,
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        }
    }

    pub fn create_text_bundle(
        prefix: String,
        text: String, 
        font_size: f32, 
        position: Vec3
    ) -> Text2dBundle {
        Text2dBundle {
            text: Text {
                sections: vec![
                    TextSection::new(
                        prefix, 
                        TextStyle {
                            font_size,
                            color : PRIMARY_COLOR,
                            ..default()
                        }
                    ),
                    TextSection::new(
                        text, 
                        TextStyle {
                            font_size,
                            color : PRIMARY_COLOR,
                            ..default()
                        }
                    )
                ],
                justify: JustifyText::Center,
                ..default()
            },
            text_anchor: Anchor::CenterLeft,
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..default()
        }
    }
}

// Separate module for creating a sprite box
pub struct SpriteBox;

impl SpriteBox {
    pub fn create_sprite_box(
        position: Vec3, 
        width: f32, 
        height: f32
    ) -> Vec<SpriteBundle> {
        let border_thickness = 2.0;

        let sprite_sizes = vec![
            Vec2::new(width, border_thickness),  // Top
            Vec2::new(width, border_thickness),  // Bottom
            Vec2::new(border_thickness, height), // Left
            Vec2::new(border_thickness, height), // Right
        ];
        
        let sprite_positions = vec![
            Vec3::new(position.x, position.y + height / 2.0, position.z), // Top
            Vec3::new(position.x, position.y - height / 2.0, position.z), // Bottom
            Vec3::new(position.x - width / 2.0, position.y, position.z),  // Left
            Vec3::new(position.x + width / 2.0, position.y, position.z),  // Right
        ];
        
        sprite_sizes.into_iter().zip(sprite_positions.into_iter())
            .map(|(size, pos)| SpriteFactory::create_sprite_bundle(size, pos))
            .collect()
    }
}