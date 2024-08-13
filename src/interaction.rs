use bevy::{
    prelude::*,
    window::PrimaryWindow,
    text::Text,
};
use crate::{
    audio::{TransientAudio, TransientAudioPallet},
    io_elements::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
};

#[derive(Component)]
pub struct Clickable {
    pub action: ClickAction,
    pub size: Vec2, // Width and height of the clickable area
    pub clicked : bool
}

impl Clickable {
    pub fn new(action: ClickAction, size: Vec2) -> Clickable {
        Clickable {
            action,
            size,
            clicked : false
        }
    }
}

#[derive(Clone)]

pub enum ClickAction {
    PlaySound(String),
    Custom(fn(&mut Commands, Entity)),
}

pub fn clickable_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut clickable_q: Query<(&GlobalTransform, &mut Clickable, Option<&mut Text>)>,
) {
    let Some(cursor_position) = get_cursor_world_position(&windows, &camera_q) else { return };

    for (transform, mut clickable, mut text) in clickable_q.iter_mut() {
        if is_cursor_within_bounds(cursor_position, transform, clickable.size) {
            if mouse_input.just_pressed(MouseButton::Left) {
                clickable.clicked = true;
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

fn trigger_clicked_audio(
    mut commands: Commands,
    mut pallet_query: Query<(Entity, &mut Clickable, &TransientAudioPallet)>,
    audio_query: Query<&TransientAudio>
) {

    for (entity, mut clickable, pallet) in pallet_query.iter_mut() {
        
        let action = clickable.action.clone();
        if clickable.clicked {
            match action {
                ClickAction::PlaySound(key) => {

                    if let Some(&audio_entity) = pallet.entities.get(&key) {
                        // Retrieve the TransientAudio component associated with the found entity
                        if let Ok(transient_audio) = audio_query.get(audio_entity) {

                            TransientAudioPallet::play_transient_audio(
                                &mut commands,
                                entity,
                                transient_audio
                            );
                        }
                    }
                },
                _ => {}
            }

            clickable.clicked = false;
        }
    }
}

pub struct InteractionPlugin<T: States + Clone + Eq + Default> {
    active_state: T,
}


impl<T: States + Clone + Eq + Default> InteractionPlugin<T> {
    pub fn new(active_state: T) -> Self {
        Self { active_state }
    }
}

impl<T: States + Clone + Eq + Default + 'static> Plugin for InteractionPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                clickable_system,
                trigger_clicked_audio
            )
            .run_if(in_state(self.active_state.clone()))
        );
    }
}