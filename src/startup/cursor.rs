use bevy::{asset::LoadState, prelude::*, render::view::RenderLayers, sprite::Anchor, utils::HashMap};
use enum_map::{Enum, EnumMap, enum_map};
use super::render::{convert_to_hdr, MainCamera};

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomCursor>()
           .add_systems(
            Startup, 
            CustomCursor::setup
        ).add_systems(
            Update, (
            CustomCursor::check_texture_loaded,
            CustomCursor::update_position,
            )
        ).add_systems(
            Update, 
            CustomCursor::update_icon
            .run_if(resource_changed::<CustomCursor>)
        );
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorMode {
    #[default]
    Pointer,
    Clicker,
    Dragger,
    Dragging
}


#[derive(Resource, Default)]
pub struct CustomCursor {
    hdr_applied: bool,
    entity: Option<Entity>,
    icons: EnumMap<CursorMode, Handle<Image>>,
    pub current_mode: CursorMode,
}

impl CustomCursor {

    fn setup(
        mut commands: Commands,
        mut window: Single<&mut Window>,
        asset_server: Res<AssetServer>,
        mut custom_cursor: ResMut<CustomCursor>,
    ) {
        window.cursor_options.visible = false;
    
        // Define cursor size
        let cursor_size = Vec2::new(15.0, 15.0);
    
        // Load the texture
        let pointer_handle: Handle<Image> = asset_server.load("textures/pointer.png");
        let clicker_handle: Handle<Image> = asset_server.load("textures/clicker.png");
        let dragger_handle: Handle<Image> = asset_server.load("textures/dragger.png");
        let dragging_handle: Handle<Image> = asset_server.load("textures/dragging.png");

        
        // Keep original spawning logic
        let cursor_entity = commands.spawn((
            Sprite {
                image: pointer_handle.clone(),
                custom_size: Some(cursor_size),
                anchor: Anchor::TopLeft,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
            RenderLayers::layer(0),
        )).id();
        
        *custom_cursor = CustomCursor {
            hdr_applied: false,
            entity: Some(cursor_entity),
            icons: enum_map![
                CursorMode::Pointer => pointer_handle.clone(),
                CursorMode::Clicker => clicker_handle.clone(),
                CursorMode::Dragger => dragger_handle.clone(),
                CursorMode::Dragging => dragging_handle.clone()
            ],
            current_mode: CursorMode::Pointer,
        };
    }

    fn check_texture_loaded(
        asset_server: Res<AssetServer>,
        mut images: ResMut<Assets<Image>>,
        mut custom_cursor: ResMut<CustomCursor>,
        mut sprites: Query<&mut Sprite>,
    ) {
        if custom_cursor.hdr_applied {
            return; // Skip if HDR already applied to all textures
        }

        // Check if we need to update any textures
        let mut all_loaded = true;
        let mut any_updated = false;
        let mut updated_handles : EnumMap<CursorMode, _> = EnumMap::default();

        // First pass - check all textures and convert to HDR if needed
        for (mode, handle) in custom_cursor.icons.iter() {
            match asset_server.get_load_state(&handle.clone()) {
                Some(LoadState::Loaded) => {
                    // Get the loaded image
                    if let Some(base_image) = images.get(handle) {
                        // Clone the image and convert to HDR
                        let base_image_clone = base_image.clone();
                        let hdr_image = convert_to_hdr(&base_image_clone, 5.0);
                    
                        // Add the HDR image to assets
                        let hdr_handle = images.add(hdr_image);
                        
                        // Store the updated handle for later assignment
                        updated_handles[mode] = Some(hdr_handle);
                        any_updated = true;
                    }
                }
                Some(LoadState::Failed(_)) => {
                    println!("Failed to load cursor texture for mode {:?}!", mode);
                    all_loaded = false;
                }
                _ => {
                    // Still loading, wait for next frame
                    all_loaded = false;
                }
            }
        }

        // Only update handles if we processed at least one texture
        if any_updated {
            // Second pass - update the handles in the custom_cursor resource
            for (mode, hdr_handle) in updated_handles.iter() {
                if let Some(hdr_handle) = hdr_handle {
                    custom_cursor.icons[mode] = hdr_handle.clone();
                }
            }
            
            // Update the active cursor sprite
            if let Some(cursor_entity) = custom_cursor.entity {
                if let Ok(mut sprite) = sprites.get_mut(cursor_entity) {
                    // Set sprite to current mode's texture
                    if let Some(handle) = updated_handles[custom_cursor.current_mode].clone() {
                        sprite.image = handle;
                    }
                }
            }
            
            // Mark as HDR applied only if all textures were loaded and converted
            custom_cursor.hdr_applied = all_loaded;
        }
    }

    fn update_position(
        window: Single<&Window>,
        custom_cursor: Res<CustomCursor>,
        mut cursor_query: Query<&mut Transform>,
        camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    ) {

        if let Some(cursor_position) = window.cursor_position() {
            // Convert screen coordinates to world coordinates
            let (camera, camera_transform) = *camera_query; 
            if let Ok(cursor_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                // Update the cursor entity position
                if let Some(cursor_entity) = custom_cursor.entity {
                    if let Ok(mut transform) = cursor_query.get_mut(cursor_entity) {                    
                        transform.translation = cursor_pos.extend(transform.translation.z);
                    }
                }
            }
        }
    }

    fn update_icon(
        custom_cursor: Res<CustomCursor>,
        mut sprites: Query<&mut Sprite>,
    ) {
        // Only proceed if we have a cursor entity
        if let Some(cursor_entity) = custom_cursor.entity {
            if let Ok(mut sprite) = sprites.get_mut(cursor_entity) {
                // Update the sprite image based on the current mode
                let current_handle = &custom_cursor.icons[custom_cursor.current_mode];
                
                // Only update if the image has changed
                if sprite.image != *current_handle {
                    sprite.image = current_handle.clone();
                }
            }
        }
    }

}