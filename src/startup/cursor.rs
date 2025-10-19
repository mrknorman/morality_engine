use bevy::{
    asset::{Assets, RenderAssetUsages}, camera::visibility::RenderLayers, image::{
        CompressedImageFormats, 
        ImageSampler, 
        ImageType
    }, prelude::*, sprite::Anchor, window::{CursorOptions, PrimaryWindow}
};

use enum_map::{
    Enum, 
    EnumMap
};

use super::render::{
    convert_to_hdr, 
    MainCamera
};

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomCursor>()
           .add_systems(
            Startup, 
            CustomCursor::setup
        ).add_systems(
            Update, 
            CustomCursor::update_position
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
    Dragging,
    PullLeft,
    PullRight
}


#[derive(Resource, Default)]
pub struct CustomCursor {
    entity: Option<Entity>,
    icons: EnumMap<CursorMode, Handle<Image>>,
    pub current_mode: CursorMode,
    pub position: Option<Vec2>
}

impl CustomCursor {

    fn setup(
        mut commands: Commands,
        mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
        mut images: ResMut<Assets<Image>>,
        mut custom_cursor: ResMut<CustomCursor>,
    ) {
        // Hide the default cursor
        cursor_options.visible = false;
        
        // Define cursor size
        let cursor_size = Vec2::new(15.0, 15.0);
        
        // Create a struct to define cursor image data
        struct CursorImageData {
            bytes: &'static [u8],
            mode: CursorMode,
        }
        
        // Define all cursor images
        let cursor_images = [
            CursorImageData { bytes: include_bytes!("content/pointer.png"), mode: CursorMode::Pointer },
            CursorImageData { bytes: include_bytes!("content/clicker.png"), mode: CursorMode::Clicker },
            CursorImageData { bytes: include_bytes!("content/dragger.png"), mode: CursorMode::Dragger },
            CursorImageData { bytes: include_bytes!("content/dragging.png"), mode: CursorMode::Dragging },
            CursorImageData { bytes: include_bytes!("content/pull_left.png"), mode: CursorMode::PullLeft },
            CursorImageData { bytes: include_bytes!("content/pull_right.png"), mode: CursorMode::PullRight },
        ];
        
        // Define parameters that can be copied
        let compressed_formats = CompressedImageFormats::all();
        let is_srgb = true;
        let asset_usage = RenderAssetUsages::default();
        let image_sampler = ImageSampler::default();
        
        // Convert and add all images to assets, then create a mapping of modes to handles
        let icons = cursor_images.into_iter().map(|data| {
            let image = Image::from_buffer(
                data.bytes,
                ImageType::Extension("png"),
                compressed_formats,
                is_srgb,
                image_sampler.clone(),
                asset_usage,
            )
            .expect("Failed to load cursor image");
            
            let hdr_image = convert_to_hdr(&image, 5.0);
            let handle = images.add(hdr_image);
            
            (data.mode, handle)
        })
        .collect::<EnumMap<CursorMode, Handle<Image>>>();
        
        // Spawn cursor entity with the default pointer image
        let cursor_entity = commands
            .spawn((
                Anchor::TOP_LEFT,
                Sprite {
                    image: icons[CursorMode::Pointer].clone(),
                    custom_size: Some(cursor_size),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 100.0)),
                RenderLayers::layer(0),
            ))
            .id();


        // Update custom cursor resource
        *custom_cursor = CustomCursor {
            entity: Some(cursor_entity),
            icons,
            current_mode: CursorMode::Pointer,
            position : None,
        };
    }

    fn update_position(
        window: Single<&Window, With<PrimaryWindow>>,
        mut custom_cursor: ResMut<CustomCursor>,
        mut cursor_query: Query<&mut Transform>,
        camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    ) {
        let (camera, camera_transform) = *camera_query;
    
        custom_cursor.position = window
            .cursor_position()
            .and_then(|cursor_screen_pos| {
                camera
                    .viewport_to_world_2d(camera_transform, cursor_screen_pos)
                    .ok()
            });
    
        if let Some(world_pos) = custom_cursor.position {
            if let Some(cursor_entity) = custom_cursor.entity {
                if let Ok(mut transform) = cursor_query.get_mut(cursor_entity) {
                    transform.translation = world_pos.extend(transform.translation.z);
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