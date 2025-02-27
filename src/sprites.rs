use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use crate::colors::PRIMARY_COLOR;

pub struct SpritePlugin;
impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app
		.register_required_components::<SpriteBox, Transform>()
        .register_required_components::<SpriteBox, Visibility>();
    }
}


pub struct SpriteFactory;

impl SpriteFactory {
    pub fn create_sprite_bundle(
        size: Vec2, 
        position: Vec3
    ) -> (Sprite, Transform) {
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

#[derive(Clone)]
pub struct SpriteBox{
    pub width : f32,
    pub height : f32
}

impl SpriteBox {
    pub fn create_sprite_box(
        position: Vec3, 
        width: f32, 
        height: f32
    ) -> Vec<(Sprite, Transform)> {
        let border_thickness = 2.0;

        let sprite_sizes = vec![
            Vec2::new(width, border_thickness),
            Vec2::new(width, border_thickness),  
            Vec2::new(border_thickness, height),
            Vec2::new(border_thickness, height), 
        ];
        
        let sprite_positions = vec![
            Vec3::new(position.x, position.y + height / 2.0, position.z), 
            Vec3::new(position.x, position.y - height / 2.0, position.z), 
            Vec3::new(position.x - width / 2.0, position.y, position.z),  
            Vec3::new(position.x + width / 2.0, position.y, position.z), 
        ];
        
        sprite_sizes.into_iter().zip(sprite_positions.into_iter())
            .map(|(size, pos)| SpriteFactory::create_sprite_bundle(size, pos))
            .collect()
    }
}


impl Component for SpriteBox{
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(

            |mut world, entity, _component_id| {       

                let size: Option<SpriteBox> = {
                    let entity_ref: EntityRef<'_> = world.entity(entity);
                    entity_ref.get::<SpriteBox>().cloned()
                };

				// Step 3: Use commands in a separate scope to spawn the entity
				if let Some(size) = size {
					let mut commands = world.commands();
					commands.entity(entity).with_children(|parent| {
                        for sprite in SpriteBox::create_sprite_box(
                            Vec3::ZERO,
                            size.width,
                            size.height,
                        ) {
                            parent.spawn(sprite);
                        }
                        
					});
				}
            }
        );
    }
}