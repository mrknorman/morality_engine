use std::hash::Hash;
use bevy::{
    prelude::*, render::primitives::Aabb, window::PrimaryWindow, math::Vec3A
};
use crate::{
    audio::{
        AudioPlugin, AudioSystemsActive, TransientAudio, TransientAudioPallet, 
    }, game_states::{
        DilemmaPhase, GameState, MainState, StateVector
    }
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InteractionSystemsActive {
    #[default]
    False,
    True
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<AudioPlugin>() {
            app.add_plugins(AudioPlugin);
        };
        app
        .init_state::<InteractionSystemsActive>()
        .add_event::<AdvanceDialogue>() // Register the event
        .add_systems(
            Update,
            activate_systems
        ).add_systems(
            Update,
            (
                clickable_system,
                pressable_system,
                trigger_audio,
                trigger_state_change,
                trigger_despawn,
                trigger_advance_dialogue
            )
            .run_if(in_state(InteractionSystemsActive::True))
        ).add_systems(
			Startup, (activate_prequisite_states)
		.run_if(in_state(InteractionSystemsActive::True)));
    }
}

fn activate_prequisite_states(        
	mut audio_state: ResMut<NextState<AudioSystemsActive>>,
) {
	audio_state.set(AudioSystemsActive::True);
}

fn activate_systems(
	mut interaction_state: ResMut<NextState<InteractionSystemsActive>>,
	pressable_query: Query<&Pressable>,
    clickable_query: Query<&Clickable>
) {

	if !pressable_query.is_empty() || !clickable_query.is_empty(){
		interaction_state.set(InteractionSystemsActive::True)
	} else {
		interaction_state.set(InteractionSystemsActive::False)
	}
}

#[derive(Event)]
pub struct AdvanceDialogue;

#[derive(Component, Clone)]
pub struct Clickable {
    pub actions: Vec<InputAction>,
    pub clicked : bool,
    actions_completed : u32
}

impl Clickable {
    pub fn new(actions: Vec<InputAction>) -> Clickable {
        Clickable {
            actions,
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
    AdvanceDialogue(String),
    Despawn
} 

pub fn clickable_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut clickable_q: Query<(
        &Aabb, 
        &GlobalTransform,
        &mut Clickable
    ), Without<TextSpan>>,
) {
    let Some(cursor_position) = get_cursor_world_position(
        &windows, &camera_q
    ) else { return };

    for (
        bound, 
        transform,
        mut clickable,
    ) in clickable_q.iter_mut() {

        if is_cursor_within_bounds(
                cursor_position, transform, &bound
            ) {
            
            if mouse_input.just_pressed(MouseButton::Left) {
                clickable.clicked = true;
            } else {
                clickable.clicked = false;
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
            } else {
                pressable.pressed = false;
            }
        }
    }
}

pub fn get_cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let cursor_position = windows.single().cursor_position()?;
    let (camera, camera_transform) = camera_q.get_single().ok()?;
    let world_position = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;
    Some(world_position.origin.truncate())
}

pub fn is_cursor_within_bounds(cursor: Vec2, transform: &GlobalTransform, aabb: &Aabb) -> bool {
    // Adjust the Aabb's center by the transform's translation
    let transformed_center = aabb.center + Vec3A::from(transform.translation());

    let bounds = (
        transformed_center.x - aabb.half_extents.x,
        transformed_center.x + aabb.half_extents.x,
        transformed_center.y - aabb.half_extents.y,
        transformed_center.y + aabb.half_extents.y,
    );

    cursor.x >= bounds.0
        && cursor.x <= bounds.1
        && cursor.y >= bounds.2
        && cursor.y <= bounds.3
}


fn trigger_audio(
    mut commands: Commands,
    mut query: Query<(Entity, Option<&mut Clickable>, Option<&mut Pressable>, &TransientAudioPallet)>,
    mut audio_query: Query<&mut TransientAudio>
) {
    fn handle_actions<T: InputActionHandler>(
        entity: Entity,
        handler: &mut T,
        pallet: &TransientAudioPallet,
        commands: &mut Commands,
        audio_query: &mut Query<&mut TransientAudio>,
    ) {

        if handler.is_triggered() {
            let actions = handler.clone_actions();
            for action in actions {
                if let InputAction::PlaySound(key) = action {
                    TransientAudioPallet::play_transient_audio(
                        entity,
                        commands,
                        pallet,
                        key,
                        audio_query
                    );
                    handler.increment();
                }
            }
            if handler.actions_completed() {
                handler.reset_trigger();
            }
        }
    }

    for (
        entity, clickable, pressable, pallet
    ) in query.iter_mut() {

        if let Some(mut handle) = clickable {
            handle_actions(
                entity, 
                &mut *handle, 
                pallet, 
                &mut commands, 
                &mut audio_query
            );
        }

        if let Some(mut handle) = pressable {
            handle_actions(
                entity, 
                &mut *handle, 
                pallet, 
                &mut commands, 
                &mut audio_query
            );
        }
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
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>
) {
    fn handle_state_change<T: InputActionHandler>(
        handler: &mut T,
        mut next_main_state: &mut ResMut<NextState<MainState>>,
        mut next_game_state: &mut ResMut<NextState<GameState>>,
        mut next_sub_state: &mut ResMut<NextState<DilemmaPhase>>,
    ) {
        if handler.is_triggered() {
            let actions = handler.clone_actions();
            for action in actions {
                if let InputAction::ChangeState(
                    state_vector
                ) = action {
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

fn trigger_advance_dialogue(
    mut event_writer: EventWriter<AdvanceDialogue>,
    mut clickable_query: Query<&mut Clickable>,
    mut pressable_query: Query<&mut Pressable>,
) {
    fn handle_actions<T: InputActionHandler>(
        handler: &mut T,
        event_writer: &mut EventWriter<AdvanceDialogue>,
    ) {
        if handler.is_triggered() {
            let actions = handler.clone_actions();
            for action in actions {
                if let InputAction::AdvanceDialogue(ref _text) = action {
                    event_writer.send(AdvanceDialogue);
                    handler.increment();
                }
            }
            if handler.actions_completed() {
                handler.reset_trigger();
            }
        }
    }

    for mut clickable in clickable_query.iter_mut() {
        handle_actions(&mut *clickable, &mut event_writer);
    }

    for mut pressable in pressable_query.iter_mut() {
        handle_actions(&mut *pressable, &mut event_writer);
    }
}

fn trigger_despawn(
    mut commands: Commands,
    mut pallet_query_mouse: Query<(
        Entity, &mut Clickable
    )>,
    mut pallet_query_keys: Query<(
        Entity, &mut Pressable
    )>,
) {
    fn handle_actions<T: InputActionHandler>(
        entity: Entity,
        handler: &mut T,
        commands: &mut Commands,
    ) {
        if handler.is_triggered() {
            let actions = handler.clone_actions();
            for action in actions {
                if let InputAction::Despawn = action {
                    // Despawn the entity
                    commands.entity(entity).despawn_recursive();
                }
            }
            if handler.actions_completed() {
                handler.reset_trigger();
            }
        }
    }

    // Handle mouse clicks
    for (
        entity, mut clickable
    ) in pallet_query_mouse.iter_mut() {
        handle_actions(entity, &mut *clickable, &mut commands);
    }

    // Handle key presses
    for (
        entity, mut pressable
    ) in pallet_query_keys.iter_mut() {
        handle_actions(entity, &mut *pressable, &mut commands);
    }
}