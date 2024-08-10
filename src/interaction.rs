use bevy::{
    prelude::*,
    window::PrimaryWindow,
    text::Text,
};
use crate::{
    audio::play_sound_once,
    io_elements::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
};

#[derive(Component)]
pub struct Clickable {
    pub action: ClickAction,
    pub size: Vec2, // Width and height of the clickable area
}

impl Clickable {
    pub fn new(action: ClickAction, size: Vec2) -> Clickable {
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
    mut clickable_q: Query<(Entity, &GlobalTransform, &Clickable, Option<&mut Text>)>,
    asset_server: Res<AssetServer>,
) {
    let Some(cursor_position) = get_cursor_world_position(&windows, &camera_q) else { return };

    for (entity, transform, clickable, mut text) in clickable_q.iter_mut() {
        if is_cursor_within_bounds(cursor_position, transform, clickable.size) {
            if mouse_input.just_pressed(MouseButton::Left) {
                trigger_click_action(&clickable.action, entity, &mut commands, &asset_server);
            } 
            
            if mouse_input.pressed(MouseButton::Left) {
                if let Some(text) = text.as_mut() {
                    update_text_color(text, PRESSED_BUTTON);
                }
            } else {
                if let Some(text) = text.as_mut() {
                    update_text_color(text, HOVERED_BUTTON);
                }
            }
        } else {
            if let Some(text) = text.as_mut() {
                update_text_color(text, NORMAL_BUTTON);
            }
        }
    }
}

fn update_text_color(text: &mut Text, color: Color) {
    for section in text.sections.iter_mut() {
        section.style.color = color;
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