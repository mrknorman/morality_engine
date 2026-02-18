use bevy::{
    asset::{Assets, RenderAssetUsages},
    camera::visibility::RenderLayers,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    input::mouse::MouseMotion,
    prelude::*,
    sprite::Anchor,
    window::{CursorOptions, PrimaryWindow},
};

use enum_map::{Enum, EnumMap};

use super::render::{convert_to_hdr, MainCamera};

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomCursor>()
            .add_systems(Startup, CustomCursor::setup)
            .add_systems(Update, CustomCursor::update_position)
            .add_systems(
                Update,
                CustomCursor::update_icon.run_if(resource_changed::<CustomCursor>),
            );
    }
}

const CURSOR_Z: f32 = 999.0;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorMode {
    #[default]
    Pointer,
    Clicker,
    Dragger,
    Dragging,
    PullLeft,
    PullRight,
}

#[derive(Resource, Default)]
pub struct CustomCursor {
    entity: Option<Entity>,
    icons: EnumMap<CursorMode, Handle<Image>>,
    pub current_mode: CursorMode,
    pub position: Option<Vec2>,
    virtual_screen_position: Option<Vec2>,
    logical_screen_position: Option<Vec2>,
    offscreen_edge: Option<OffscreenEdge>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OffscreenEdge {
    Left,
    Right,
    Top,
    Bottom,
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
            CursorImageData {
                bytes: include_bytes!("content/pointer.png"),
                mode: CursorMode::Pointer,
            },
            CursorImageData {
                bytes: include_bytes!("content/clicker.png"),
                mode: CursorMode::Clicker,
            },
            CursorImageData {
                bytes: include_bytes!("content/dragger.png"),
                mode: CursorMode::Dragger,
            },
            CursorImageData {
                bytes: include_bytes!("content/dragging.png"),
                mode: CursorMode::Dragging,
            },
            CursorImageData {
                bytes: include_bytes!("content/pull_left.png"),
                mode: CursorMode::PullLeft,
            },
            CursorImageData {
                bytes: include_bytes!("content/pull_right.png"),
                mode: CursorMode::PullRight,
            },
        ];

        // Define parameters that can be copied
        let compressed_formats = CompressedImageFormats::all();
        let is_srgb = true;
        let asset_usage = RenderAssetUsages::default();
        let image_sampler = ImageSampler::default();

        // Convert and add all images to assets, then create a mapping of modes to handles
        let icons = cursor_images
            .into_iter()
            .map(|data| {
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
                Transform::from_translation(Vec3::new(0.0, 0.0, CURSOR_Z)),
                // Keep cursor on the world layer so CRT/scanline post-processing still affects it.
                RenderLayers::layer(0),
            ))
            .id();

        // Update custom cursor resource
        *custom_cursor = CustomCursor {
            entity: Some(cursor_entity),
            icons,
            current_mode: CursorMode::Pointer,
            position: None,
            virtual_screen_position: None,
            logical_screen_position: None,
            offscreen_edge: None,
        };
    }

    fn update_position(
        window: Single<&Window, With<PrimaryWindow>>,
        mut mouse_motion_events: MessageReader<MouseMotion>,
        mut custom_cursor: ResMut<CustomCursor>,
        mut cursor_query: Query<&mut Transform>,
        camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    ) {
        let (camera, camera_transform) = *camera_query;

        let motion_delta = mouse_motion_events
            .read()
            .fold(Vec2::ZERO, |acc, event| acc + event.delta);

        let max_screen_x = (window.resolution.width() - 1.0).max(0.0);
        let max_screen_y = (window.resolution.height() - 1.0).max(0.0);
        let clamp_to_window = |position: Vec2| -> Vec2 {
            Vec2::new(
                position.x.clamp(0.0, max_screen_x),
                position.y.clamp(0.0, max_screen_y),
            )
        };

        let nearest_edge = |position: Vec2| -> OffscreenEdge {
            let left = position.x;
            let right = max_screen_x - position.x;
            let top = position.y;
            let bottom = max_screen_y - position.y;
            let min_distance = left.min(right).min(top).min(bottom);

            if min_distance == left {
                OffscreenEdge::Left
            } else if min_distance == right {
                OffscreenEdge::Right
            } else if min_distance == top {
                OffscreenEdge::Top
            } else {
                OffscreenEdge::Bottom
            }
        };

        let project_to_edge = |position: Vec2, edge: OffscreenEdge| -> Vec2 {
            match edge {
                OffscreenEdge::Left => Vec2::new(0.0, position.y.clamp(0.0, max_screen_y)),
                OffscreenEdge::Right => {
                    Vec2::new(max_screen_x, position.y.clamp(0.0, max_screen_y))
                }
                OffscreenEdge::Top => Vec2::new(position.x.clamp(0.0, max_screen_x), 0.0),
                OffscreenEdge::Bottom => {
                    Vec2::new(position.x.clamp(0.0, max_screen_x), max_screen_y)
                }
            }
        };

        let detect_offscreen_edge =
            |position: Vec2, previous: Option<OffscreenEdge>| -> OffscreenEdge {
                let left_overflow = (-position.x).max(0.0);
                let right_overflow = (position.x - max_screen_x).max(0.0);
                let top_overflow = (-position.y).max(0.0);
                let bottom_overflow = (position.y - max_screen_y).max(0.0);

                let mut edge = previous.unwrap_or(OffscreenEdge::Right);
                let mut max_overflow = -1.0_f32;

                let candidates = [
                    (OffscreenEdge::Left, left_overflow),
                    (OffscreenEdge::Right, right_overflow),
                    (OffscreenEdge::Top, top_overflow),
                    (OffscreenEdge::Bottom, bottom_overflow),
                ];

                for (candidate_edge, overflow) in candidates {
                    if overflow > max_overflow {
                        max_overflow = overflow;
                        edge = candidate_edge;
                    }
                }

                if max_overflow > 0.0 {
                    edge
                } else {
                    previous.unwrap_or_else(|| nearest_edge(position))
                }
            };

        let next_screen_position = if let Some(real_cursor_screen_pos) = window.cursor_position() {
            let clamped = clamp_to_window(real_cursor_screen_pos);
            custom_cursor.virtual_screen_position = Some(clamped);
            custom_cursor.logical_screen_position = Some(clamped);
            custom_cursor.offscreen_edge = None;
            Some(clamped)
        } else {
            let next_virtual = custom_cursor
                .virtual_screen_position
                .or(custom_cursor.logical_screen_position)
                .map(|position| position + motion_delta);

            custom_cursor.virtual_screen_position = next_virtual;

            let projected = next_virtual.map(|position| {
                let edge = detect_offscreen_edge(position, custom_cursor.offscreen_edge);
                custom_cursor.offscreen_edge = Some(edge);
                project_to_edge(position, edge)
            });

            custom_cursor.logical_screen_position = projected;
            projected
        };

        let previous_world_position = custom_cursor.position;
        custom_cursor.position = next_screen_position.and_then(|screen_position| {
            camera
                .viewport_to_world_2d(camera_transform, screen_position)
                .ok()
                .or(previous_world_position)
        });

        if let Some(world_pos) = custom_cursor.position {
            if let Some(cursor_entity) = custom_cursor.entity {
                if let Ok(mut transform) = cursor_query.get_mut(cursor_entity) {
                    transform.translation = world_pos.extend(transform.translation.z);
                }
            }
        }
    }

    fn update_icon(custom_cursor: Res<CustomCursor>, mut sprites: Query<&mut Sprite>) {
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
