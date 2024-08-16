use bevy::{
    prelude::*,
    window::PrimaryWindow,
    text::Text,
};
use crate::{
    audio::{
        TransientAudio, 
        TransientAudioPallet, 
        AudioPlugin
    },
    game_states::{
        StateVector,
        MainState,
        GameState,
        SubState
    }
};

pub const NORMAL_BUTTON: Color = Color::srgb(1.0, 1.0, 1.0);
pub const HOVERED_BUTTON: Color = Color::srgb(0.0, 1.0, 1.0);
pub const PRESSED_BUTTON: Color = Color::srgb(1.0, 1.0, 0.0);

#[derive(Component)]
pub struct Clickable {
    pub actions: Vec<InputAction>,
    pub size: Vec2, // Width and height of the clickable area
    pub clicked : bool,
    actions_completed : u32
}

impl Clickable {
    pub fn new(actions: Vec<InputAction>, size: Vec2) -> Clickable {
        Clickable {
            actions,
            size,
            clicked : false,
            actions_completed : 0
        }
    }
}

#[derive(Component)]
pub struct Pressable {
    pub keys : Vec<KeyCode>,
    pub actions: Vec<InputAction>,
    pub pressed : bool,
    actions_completed : u32
}

impl Pressable {
    pub fn new(
        keys : Vec<KeyCode>,
        actions: Vec<InputAction>,
    ) -> Pressable {

        Pressable {
            keys,
            actions,
            pressed : false,
            actions_completed : 0
        }
    }
}

#[derive(Clone)]

pub enum InputAction {
    PlaySound(String),
    ChangeState(StateVector),
    Custom(fn(&mut Commands, Entity)),
}

pub fn clickable_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut clickable_q: Query<(
        &GlobalTransform, 
        &mut Clickable, Option<&mut Text>
    )>,
) {
    let Some(cursor_position) = get_cursor_world_position(
        &windows, &camera_q
    ) else { return };

    for (
        transform, 
        mut clickable,
        mut text
    ) in clickable_q.iter_mut() {

        if is_cursor_within_bounds(
                cursor_position, transform, clickable.size
            ) {
            
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

pub fn pressable_system (
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut pressable_q: Query<&mut Pressable>
) {

    for mut pressable in pressable_q.iter_mut() {

        for key in pressable.keys.clone() {
            if keyboard_input.just_pressed(key) {
                pressable.pressed = true;
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
    let (
        camera, 
        camera_transform
    ) = camera_q.get_single().ok()?;
    let world_position = camera.viewport_to_world(
        camera_transform, 
        cursor_position
    )?;
    Some(world_position.origin.truncate())
}

fn is_cursor_within_bounds(
        cursor: Vec2, 
        transform: &GlobalTransform, 
        size: Vec2
    ) -> bool {

    let entity_position = transform.translation().truncate();
    let half_size = size / 2.0;
    let bounds = (
        entity_position.x - half_size.x,
        entity_position.x + half_size.x,
        entity_position.y - half_size.y,
        entity_position.y + half_size.y,
    );

       cursor.x >= bounds.0 
    && cursor.x <= bounds.1 
    && cursor.y >= bounds.2
    && cursor.y <= bounds.3
}

fn trigger_audio(
    mut commands: Commands,
    mut pallet_query_mouse: Query<(Entity, &mut Clickable, &TransientAudioPallet)>,
    mut pallet_query_keys: Query<(Entity, &mut Pressable, &TransientAudioPallet)>,
    mut audio_query: Query<&mut TransientAudio>
) {
    fn handle_actions<T: InputActionHandler>(
        entity: Entity,
        handler: &mut T,
        pallet: &TransientAudioPallet,
        mut commands: &mut Commands,
        audio_query: &mut Query<&mut TransientAudio>,
    ) {
        if handler.is_triggered() {
            let actions = handler.clone_actions();
            for action in actions {
                if let InputAction::PlaySound(key) = action {
                    if let Some(&audio_entity) = pallet.entities.get(&key) {
                        if let Ok(mut transient_audio) = audio_query.get_mut(audio_entity) {
                            TransientAudioPallet::play_transient_audio(
                                &mut commands,
                                entity,
                                &mut transient_audio,
                            );
                        }
                    }

                    handler.increment();
                }
            }
            if handler.actions_completed() {
                handler.reset_trigger();
            }
        }
    }

    for (entity, mut clickable, pallet) in pallet_query_mouse.iter_mut() {
        handle_actions(
            entity, &mut *clickable, pallet, &mut commands, &mut audio_query
        );
    }

    for (entity, mut pressable, pallet) in pallet_query_keys.iter_mut() {
        handle_actions(entity, &mut *pressable, pallet, &mut commands, &mut audio_query);
    }
}

trait InputActionHandler {
    fn is_triggered(&self) -> bool;
    fn clone_actions(&self) -> Vec<InputAction>;
    fn actions_completed(&self) -> bool;
    fn reset_trigger(&mut self);
    fn increment(&mut self);
}

impl InputActionHandler for Clickable {
    fn is_triggered(&self) -> bool {
        self.clicked
    }

    fn clone_actions(&self) -> Vec<InputAction> {
        self.actions.clone()
    }

    fn actions_completed(&self) -> bool {
        self.actions_completed >= self.actions.len() as u32
    }

    fn reset_trigger(&mut self) {
        self.clicked = false;
        self.actions_completed = 0;
    }

    fn increment(&mut self) {
        self.actions_completed += 1;
    }
}

impl InputActionHandler for Pressable {
    fn is_triggered(&self) -> bool {
        self.pressed
    }

    fn clone_actions(&self) -> Vec<InputAction> {
        self.actions.clone()
    }

    fn actions_completed(&self) -> bool {
        self.actions_completed >= self.actions.len() as u32
    }

    fn reset_trigger(&mut self) {
        self.pressed = false;
        self.actions_completed = 0;
    }

    fn increment(&mut self) {
        self.actions_completed += 1;
    }
}

fn trigger_state_change(
    mut clickable_query: Query<&mut Clickable>,
    mut pressable_query: Query<&mut Pressable>,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<SubState>>
) {
    fn handle_state_change<T: InputActionHandler>(
        handler: &mut T,
        mut next_main_state: &mut ResMut<NextState<MainState>>,
        mut next_game_state: &mut ResMut<NextState<GameState>>,
        mut next_sub_state: &mut ResMut<NextState<SubState>>,
    ) {
        if handler.is_triggered() {
            let actions = handler.clone_actions();
            for action in actions {
                if let InputAction::ChangeState(state_vector) = action {
                    state_vector.set_state(
                        &mut next_main_state,
                        &mut next_game_state,
                        &mut next_sub_state,
                    );
                }
            }
            if handler.actions_completed() {
                handler.reset_trigger();
            }
        }
    }

    for mut clickable in clickable_query.iter_mut() {
        handle_state_change(
            &mut *clickable,
            &mut next_main_state,
            &mut next_game_state,
            &mut next_sub_state,
        );
    }

    for mut pressable in pressable_query.iter_mut() {
        handle_state_change(
            &mut *pressable,
            &mut next_main_state,
            &mut next_game_state,
            &mut next_sub_state,
        );
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
                pressable_system,
                trigger_audio,
                trigger_state_change
            )
            .run_if(in_state(self.active_state.clone()))
        ).add_plugins(
            AudioPlugin::new(
                self.active_state.clone()
            )
        );
    }
}