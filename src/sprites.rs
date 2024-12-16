use bevy::prelude::*;
use crate::colors::PRIMARY_COLOR;

// Separate module for general sprite creation functionality
pub struct SpriteFactory;

impl SpriteFactory {
    pub fn create_sprite_bundle(size: Vec2, position: Vec3) -> (Sprite, Transform) {
        (
            Sprite{
                custom_size: Some(size),
                color : PRIMARY_COLOR,
                ..default()
            },
            Transform::from_xyz(
                position.x,
                position.y, 
                position.z
            )
        )
    }
}

// Separate module for creating a sprite box
pub struct SpriteBox;

impl SpriteBox {
    pub fn create_sprite_box(
        position: Vec3, 
        width: f32, 
        height: f32
    ) -> Vec<(Sprite, Transform)> {
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