use std::hash::Hash;
use enum_map::{
    Enum, 
    EnumArray, 
    EnumMap
};
use bevy::{
    math::Vec3A, 
    prelude::*, 
    render::primitives::Aabb, 
    window::PrimaryWindow
};
use crate::{
    audio::{
        AudioPlugin, 
        AudioSystemsActive, 
        DilatableAudio, 
        TransientAudio, 
        TransientAudioPallet 
    }, 
    dilemma::lever::{
        Lever, 
        LeverState
    }, 
    game_states::{
        DilemmaPhase, 
        GameState, 
        MainState, 
        StateVector
    }, 
    time::Dilation
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
            activate_systems::<InputAction>
        ).add_systems(
            Update,
            (
                clickable_system::<InputAction>,
                pressable_system::<InputAction>,
                trigger_audio,
                trigger_state_change,
                trigger_despawn,
                trigger_advance_dialogue,
                trigger_lever_state_change
            )
            .run_if(in_state(InteractionSystemsActive::True))
        ).add_systems(
			Startup, (activate_prerequisite_states)
		.run_if(in_state(InteractionSystemsActive::True)));
    }
}

fn activate_prerequisite_states(        
	mut audio_state: ResMut<NextState<AudioSystemsActive>>,
) {
	audio_state.set(AudioSystemsActive::True);
}

fn activate_systems<T: Send + Sync + 'static>(
    mut state: ResMut<NextState<InteractionSystemsActive>>,
    query: Query<(), Or<(
        With<Pressable<T>>,
        With<Clickable<T>>
    )>>
) {
    if !query.is_empty() {
        state.set(InteractionSystemsActive::True)
    } else {
        state.set(InteractionSystemsActive::False)
    }
}

#[derive(Event)]
pub struct AdvanceDialogue;

#[derive(Component)]
pub struct Clickable<T> where T: Send, T: Sync {
    pub actions: Vec<T>,
    pub triggered: bool,
    actions_completed: u32
}

#[derive(Component)]
pub struct Pressable<T> where T: Send, T: Sync {
    pub keys: Vec<KeyCode>,
    pub actions: Vec<T>,
    pub triggered: bool,
    actions_completed: u32
}

#[derive(Component)]
pub struct ClickablePong<T> {
    current_index: usize,
    direction: PongDirection,
    action_vector : Vec<Vec<T>>,
    pub actions: Vec<T>,
    pub triggered : bool,
    actions_completed : u32
}


impl<T> Clickable<T> where T: Send, T: Sync {
    pub fn new(actions: Vec<T>) -> Self {
        Clickable {
            actions,
            triggered : false,
            actions_completed : 0
        }
    }
}

impl<T> Pressable<T> where T: Send, T: Sync {
    pub fn new(keys: Vec<KeyCode>, actions: Vec<T>) -> Self {
        Self {
            keys,
            actions,
            triggered: false,
            actions_completed: 0
        }
    }
}

impl<T: Clone> ClickablePong<T> {
    pub fn new(action_vector: Vec<Vec<T>>) -> Self {
        let actions = action_vector[0].clone();
        Self {
            current_index: 0,
            direction: PongDirection::Forward,
            action_vector,
            actions,
            triggered: false,
            actions_completed: 0,
        }
    }
}


#[derive(Clone, Copy)]
enum PongDirection {
    Forward,
    Backward,
}

#[derive(Clone)]
pub enum InputAction {
    PlaySound(String),
    ChangeState(StateVector),
    AdvanceDialogue(String),
    ChangeLeverState(LeverState),
    Despawn
}

pub fn clickable_system<T: Send + Sync + 'static>(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut clickable_q: Query<(
        &Aabb, 
        &GlobalTransform,
        &mut Clickable<T>
    ), Without<TextSpan>>,
    mut clickable_p_q: Query<(
        &Aabb, 
        &GlobalTransform,
        &mut ClickablePong<T>
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
                clickable.triggered = true;
            } else {
                clickable.triggered = false;
            }
        }
    }

    for (
        bound, 
        transform,
        mut clickable,
    ) in clickable_p_q.iter_mut() {

        if is_cursor_within_bounds(
                cursor_position, transform, &bound
            ) {
            
            if mouse_input.just_pressed(MouseButton::Left) {
                clickable.triggered = true;
            } else {
                clickable.triggered = false;
            }
        }
    }
}

pub fn pressable_system<T: Send + Sync + 'static>(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut pressable_q: Query<&mut Pressable<T>>
) {

    for mut pressable in pressable_q.iter_mut() {

        for key in pressable.keys.clone() {
            if keyboard_input.just_pressed(key) {
                pressable.triggered = true;
            } else {
                pressable.triggered = false;
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

trait InputActionContainer<T> {
    fn get_triggered(&self) -> bool;
    fn get_actions(&self) -> &Vec<T>;
    fn get_actions_completed(&self) -> u32;
    fn set_triggered(&mut self, value: bool);
    fn set_actions_completed(&mut self, value: u32);
}

macro_rules! impl_input_action_container {
    ($type:ty, $action_type:ident) => {
        impl<$action_type: Send + Sync + 'static> InputActionContainer<$action_type> for $type {
            fn get_triggered(&self) -> bool {
                self.triggered
            }
            fn get_actions(&self) -> &Vec<$action_type> {
                &self.actions
            }
            fn get_actions_completed(&self) -> u32 {
                self.actions_completed
            }
            fn set_triggered(&mut self, value: bool) {
                self.triggered = value;
            }
            fn set_actions_completed(&mut self, value: u32) {
                self.actions_completed = value;
            }
        }
    };
}

// Apply the macro to both structs
impl_input_action_container!(Clickable<T>, T);
impl_input_action_container!(Pressable<T>, T);
impl_input_action_container!(ClickablePong<T>, T);

trait InputActionHandler<T: Clone> {
    fn is_triggered(&self) -> bool;
    fn clone_actions(&self) -> Vec<T>;
    fn actions_completed(&self) -> bool;
    fn reset_trigger(&mut self);
    fn increment(&mut self);
}

impl<T> InputActionHandler<T> for Clickable<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn is_triggered(&self) -> bool {
        self.get_triggered()
    }

    fn clone_actions(&self) -> Vec<T> {
        self.get_actions().clone() // Ensure `get_actions` returns `Vec<T>`
    }

    fn actions_completed(&self) -> bool {
        self.get_actions_completed() >= self.get_actions().len() as u32
    }

    fn reset_trigger(&mut self) {
        self.set_triggered(false);
        self.set_actions_completed(0);
    }

    fn increment(&mut self) {
        self.set_actions_completed(self.get_actions_completed() + 1);
    }
}

impl<T> InputActionHandler<T> for Pressable <T>
where
    T: Clone + Send + Sync + 'static,
{
    fn is_triggered(&self) -> bool {
        self.get_triggered()
    }

    fn clone_actions(&self) -> Vec<T> {
        self.get_actions().clone()
    }

    fn actions_completed(&self) -> bool {
        self.get_actions_completed() >= self.get_actions().len() as u32
    }

    fn reset_trigger(&mut self) {
        self.set_triggered(false);
        self.set_actions_completed(0);
    }

    fn increment(&mut self) {
        self.set_actions_completed(self.get_actions_completed() + 1);
    }
}

impl<T> InputActionHandler<T> for ClickablePong <T>
where
    T: Clone + Send + Sync + 'static,
{
    fn is_triggered(&self) -> bool {
        self.get_triggered()
    }

    fn clone_actions(&self) -> Vec<T> {
        self.get_actions().clone()
    }

    fn actions_completed(&self) -> bool {
        self.get_actions_completed() >= self.get_actions().len() as u32      
    }

    fn reset_trigger(&mut self) {
        self.set_triggered(false);
        self.set_actions_completed(0);

        match self.direction {
            PongDirection::Forward => {
                if self.current_index >= self.action_vector.len() - 1 {
                    self.direction = PongDirection::Backward;
                    self.current_index = self.action_vector.len().saturating_sub(2);
                } else {
                    self.current_index += 1;
                }
            }
            PongDirection::Backward => {
                if self.current_index == 0 {
                    self.direction = PongDirection::Forward;
                    self.current_index = 1;
                } else {
                    self.current_index -= 1;
                }
            }
        } 

        self.actions = self.action_vector[self.current_index].clone();
    }

    fn increment(&mut self) {
        self.set_actions_completed(self.get_actions_completed() + 1);
    }
}


macro_rules! handle_all_actions {
    ($handler:expr => {
        $($variant:ident $( ( $($binding:pat),* ) )? => $body:block),* $(,)?
    }) => {{
        use InputAction::*;
        let handler = $handler; // shadow the name
        if handler.is_triggered() {
            let actions: Vec<InputAction> = handler.clone_actions();

            for action in actions {
                match action {
                    $(
                        $variant $( ( $($binding),* ) )? => { // Match both unit and tuple variants
                            handler.increment();
                            $body
                        }
                    ),*
                    _ => {}
                }
            }
            if handler.actions_completed() {
                handler.reset_trigger();
            }
        }
    }}
}


macro_rules! handle_triggers {
    ($clickable:expr, $pressable:expr, $pong:expr, $handle_ident:ident => $block:block) => {{
        if let Some(mut $handle_ident) = $clickable {
            let $handle_ident = &mut *$handle_ident;
            $block
        }
        if let Some(mut $handle_ident) = $pressable {
            let $handle_ident = &mut *$handle_ident;
            $block
        }
        if let Some(mut $handle_ident) = $pong {
            let $handle_ident = &mut *$handle_ident;
            $block
        }
    }};
}


fn trigger_audio(
    mut commands: Commands,
    mut query: Query<(
        Entity, 
        Option<&mut Clickable<InputAction>>, 
        Option<&mut Pressable<InputAction>>, 
        Option<&mut ClickablePong<InputAction>>, 
        &TransientAudioPallet
    )>,
    mut audio: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation : Res<Dilation>,
) {
    for (entity, clickable, pressable, pong, pallet) in &mut query {
        handle_triggers!(clickable, pressable, pong, handle => {
            handle_all_actions!(handle => {
                PlaySound(key) => {
                    TransientAudioPallet::play_transient_audio(
                        entity,
                        &mut commands,
                        pallet,
                        key,
                        dilation.0,
                        &mut audio,
                    );
                },
            });
        });
    }
}

fn trigger_state_change(
    mut query: Query<(
        Entity, 
        Option<&mut Clickable<InputAction>>, 
        Option<&mut Pressable<InputAction>>, 
        Option<&mut ClickablePong<InputAction>>
    )>,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>
) {
    for (_, clickable, pressable, pong) in &mut query {
        handle_triggers!(clickable, pressable, pong, handle => {
            handle_all_actions!(handle => {
                ChangeState(state_vector) => {
                    state_vector.set_state(
                        &mut next_main_state,
                        &mut next_game_state,
                        &mut next_sub_state,
                    );
                }
            });
        });
    }
}

fn trigger_advance_dialogue(
        mut query: Query<(
            Entity, 
            Option<&mut Clickable<InputAction>>, 
            Option<&mut Pressable<InputAction>>, 
            Option<&mut ClickablePong<InputAction>>
        )>,
        mut event_writer: EventWriter<AdvanceDialogue>
    ) {

    fn send_dialogue_event(event_writer: &mut EventWriter<AdvanceDialogue>) {
        event_writer.send(AdvanceDialogue);
    }

    for (_, clickable, pressable, pong) in &mut query {
        handle_triggers!(clickable, pressable, pong, handle => {
            handle_all_actions!(handle => {
                AdvanceDialogue(_) => {
                    send_dialogue_event(&mut event_writer);
                }
            });
        });
    }
}

fn trigger_despawn(
    mut commands: Commands,
    mut query: Query<(
        Entity, 
        Option<&mut Clickable<InputAction>>, 
        Option<&mut Pressable<InputAction>>, 
        Option<&mut ClickablePong<InputAction>>
    )>
) {
    for (entity, clickable, pressable, pong) in &mut query {
        handle_triggers!(clickable, pressable, pong, handle => {
            handle_all_actions!(handle => {
                Despawn => {
                    commands.entity(entity).despawn_recursive();
                }
            });
        });
    }
}

fn trigger_lever_state_change(
    mut lever: ResMut<Lever>,
    mut query: Query<(
        Entity, 
        Option<&mut Clickable<InputAction>>, 
        Option<&mut Pressable<InputAction>>, 
        Option<&mut ClickablePong<InputAction>>
    )>
) {

    let mut lever_state = lever.0;
    for (_, clickable, pressable, pong) in &mut query {
        handle_triggers!(clickable, pressable, pong, handle => {
            handle_all_actions!(handle => {
                ChangeLeverState(new_lever_state)  => {
                    lever_state = new_lever_state;
                }
            });
        });
    }
    lever.0 = lever_state;
}

#[derive(Component)]
pub struct ActionPallet<T: Enum + EnumArray<Vec<InputAction>>>(pub EnumMap<T, Vec<InputAction>>);