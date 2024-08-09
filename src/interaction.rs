use bevy::prelude::*;
use crate::audio::play_sound_once;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct Clickable {
    pub action: ClickAction,
    pub size: Vec2, // Width and height of the clickable area
}

impl Clickable{
    pub fn new(action : ClickAction, size: Vec2) -> Clickable {
        Clickable {
            action,
            size
        }
    }
}

pub enum ClickAction {
    PlaySound(&'static str),
    Custom(fn(&mut Commands, Entity)),
}

pub fn clickable_system(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    clickable_q: Query<(Entity, &GlobalTransform, &Clickable)>,
    asset_server: Res<AssetServer>,
) {
    let Some(cursor_position) = get_cursor_world_position(&windows, &camera_q) else { return };

    if mouse_input.just_pressed(MouseButton::Left) {
        for (entity, transform, clickable) in clickable_q.iter() {
            if is_cursor_within_bounds(cursor_position, transform, clickable.size) {
                trigger_click_action(&clickable.action, entity, &mut commands, &asset_server);
                break;
            }
        }
    }
}

fn get_cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let cursor_position = windows.single().cursor_position()?;
    let (camera, camera_transform) = camera_q.get_single().ok()?;
    let world_position = camera.viewport_to_world(camera_transform, cursor_position)?;
    Some(world_position.origin.truncate())
}

fn is_cursor_within_bounds(cursor: Vec2, transform: &GlobalTransform, size: Vec2) -> bool {
    let entity_position = transform.translation().truncate();
    let half_size = size / 2.0;
    let bounds = (
        entity_position.x - half_size.x,
        entity_position.x + half_size.x,
        entity_position.y - half_size.y,
        entity_position.y + half_size.y,
    );

    cursor.x >= bounds.0 && cursor.x <= bounds.1 && cursor.y >= bounds.2 && cursor.y <= bounds.3
}

fn trigger_click_action(
    action: &ClickAction,
    entity: Entity,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    match action {
        ClickAction::PlaySound(sound_path) => {
            play_sound_once(sound_path, commands, asset_server);
        }
        ClickAction::Custom(func) => func(commands, entity),
    }
}
