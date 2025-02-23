use std::hash::Hash;
use std::fmt;
use enum_map::{
    Enum, 
    EnumArray,
    EnumMap
};
use bevy::{
    ecs::component::StorageType, math::Vec3A, prelude::*, render::primitives::Aabb, window::PrimaryWindow
};
use crate::{
    audio::{
        AudioPlugin,
        AudioSystemsActive,
        DilatableAudio,
        TransientAudio,
        TransientAudioPallet,
    }, 
    dialogue::dialogue::DialogueActions, 
    dilemma::{
        lever::{
            Lever,
            LeverState,
        },
        DilemmaActions, 
        LeverActions,
    }, game_states::{
        DilemmaPhase,
        GameState,
        MainState,
        StateVector,
    }, 
    loading::LoadingActions, 
    menu::MenuActions, 
    time::Dilation, 
    train::TrainActions
};


#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InteractionSystem {
    Clickable,
    Pressable,
    Audio,
    AdvanceDialogue,
    LeverChange,
    Debug,
    Pong,
    StateChange, // Stage change: second-to-last.
    Despawn,     // Despawn: last.
}

macro_rules! system_entry {
    ($system:expr, $label:expr) => {
        IntoSystem::into_system($system).in_set($label)
    };
    ($system:expr, $label:expr, after: $after:expr) => {
        IntoSystem::into_system($system).in_set($label).after($after)
    };
}

macro_rules! register_interaction_systems {
    ($app:expr, $enum_type:ty) => {
        $app.add_systems(
            Update,
            (
                system_entry!(clickable_system::<$enum_type>, InteractionSystem::Clickable),
                system_entry!(pressable_system::<$enum_type>, InteractionSystem::Pressable, after: InteractionSystem::Clickable),
                system_entry!(trigger_audio::<$enum_type>, InteractionSystem::Audio, after: InteractionSystem::Pressable),
                system_entry!(trigger_advance_dialogue::<$enum_type>, InteractionSystem::AdvanceDialogue, after: InteractionSystem::Audio),
                system_entry!(trigger_lever_state_change::<$enum_type>, InteractionSystem::LeverChange, after: InteractionSystem::AdvanceDialogue),
                system_entry!(trigger_debug_print::<$enum_type>, InteractionSystem::Debug, after: InteractionSystem::LeverChange),
                system_entry!(update_pong::<$enum_type>, InteractionSystem::Pong, after: InteractionSystem::Debug),
                system_entry!(trigger_state_change::<$enum_type>, InteractionSystem::StateChange, after: InteractionSystem::Pong),
                system_entry!(trigger_despawn::<$enum_type>, InteractionSystem::Despawn, after: InteractionSystem::StateChange),
            )
        );
    };
}pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AudioPlugin>() {
            app.add_plugins(AudioPlugin);
        }
        app.add_event::<AdvanceDialogue>()
        .add_systems(Startup, activate_prerequisite_states);

        register_interaction_systems!(app, MenuActions);
        register_interaction_systems!(app, LoadingActions);
        register_interaction_systems!(app, DilemmaActions);
        register_interaction_systems!(app, DialogueActions);
        register_interaction_systems!(app, LeverActions);
        register_interaction_systems!(app, TrainActions);


    }
}

fn activate_prerequisite_states(
    mut audio_state: ResMut<NextState<AudioSystemsActive>>,
) {
    audio_state.set(AudioSystemsActive::True);
}

#[derive(Event)]
pub struct AdvanceDialogue;

#[derive(Component)]
pub struct Clickable<T>
where
    T: Copy + Send + Sync,
{
    /// Keys used to look up actions in the ActionPallet.
    pub actions: Vec<T>,
    pub triggered: bool,
}

pub struct KeyMapping<T>
where
    T: Copy + Send + Sync,
{
    pub keys: Vec<KeyCode>,
    pub actions: Vec<T>,
    pub allow_repeated_activation: bool
}

#[derive(Component)]
#[require(InteractionState)]
pub struct Pressable<T>
where
    T: Copy + Send + Sync,
{
    /// Each tuple maps a group of keys to its associated actions.
    pub mappings: Vec<KeyMapping<T>>,
    /// Optionally store which mapping (if any) was triggered this frame.
    pub triggered_mapping: Option<usize>,
}

impl<T> Pressable<T>
where
    T: Copy + Send + Sync,
{
    pub fn new(mappings: Vec<KeyMapping<T>>) -> Self {
        Self {
            mappings,
            triggered_mapping: None,
        }
    }
}

#[derive(Component)]
pub struct InteractionState(pub usize);

impl Default for InteractionState {
    fn default() -> Self {
        Self(0)
    }
}


#[derive(Clone)]
pub struct ClickablePong<T> {
    /// The ping–pong index and cycle state.
    pub direction: PongDirection,
    /// A vector of key sets (each a Vec<T>) to cycle through.
    pub action_vector: Vec<Vec<T>>
}

impl<T> Component for ClickablePong<T> 
where
    T: Copy + Send + Sync + 'static,
{
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(
        hooks: &mut bevy::ecs::component::ComponentHooks,
    ) {
        hooks.on_insert(  
            |mut world, entity, _component_id| {
                if let Some(pong) = world.entity(entity).get::<ClickablePong<T>>().cloned() {
                    world.commands().entity(entity).insert((
                        Clickable::new(pong.action_vector[0].clone()),
                        InteractionState::default(),
                    ));
                }
            }
        );
    }
}

impl<T> Clickable<T>
where
    T: Copy + Send + Sync,
{
    pub fn new(actions: Vec<T>) -> Self {
        Clickable {
            actions,
            triggered: false,
        }
    }
}

impl<T: Clone> ClickablePong<T> {
    pub fn new(action_vector: Vec<Vec<T>>) -> Self {
        Self {
            direction: PongDirection::Forward,
            action_vector,
        }
    }
}

#[derive(Clone, Copy)]
pub enum PongDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    PlaySound(String),
    ChangeState(StateVector),
    AdvanceDialogue(String),
    ChangeLeverState(LeverState),
    Despawn,
    #[allow(unused)]
    Print(String), 
}

impl fmt::Display for InputAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         write!(f, "{:?}", self)
    }
}

/// Utility function for cursor handling.
pub fn get_cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.get_single().ok()?;
    let cursor_position = window.cursor_position()?;
    let (camera, camera_transform) = camera_q.get_single().ok()?;
    let world_position = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;
    Some(world_position.origin.truncate())
}

pub fn is_cursor_within_bounds(cursor: Vec2, transform: &GlobalTransform, aabb: &Aabb) -> bool {
    let transformed_center = aabb.center + Vec3A::from(transform.translation());
    let bounds = (
        transformed_center.x - aabb.half_extents.x,
        transformed_center.x + aabb.half_extents.x,
        transformed_center.y - aabb.half_extents.y,
        transformed_center.y + aabb.half_extents.y,
    );
    cursor.x >= bounds.0 &&
    cursor.x <= bounds.1 &&
    cursor.y >= bounds.2 &&
    cursor.y <= bounds.3
}

/// The InputActionHandler trait is used by Clickable and Pressable
/// to look up their actions in the ActionPallet.
pub trait InputActionHandler<K>
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Send + Sync,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    fn is_triggered(&self) -> bool;
    fn clone_actions_from_pallet(&self, pallet: &ActionPallet<K>) -> Vec<InputAction>;
}

impl<K> InputActionHandler<K> for Clickable<K>
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    fn is_triggered(&self) -> bool {
        self.triggered
    }
    fn clone_actions_from_pallet(&self, pallet: &ActionPallet<K>) -> Vec<InputAction> {
        let mut actions = Vec::new();
        for key in &self.actions {
            actions.extend_from_slice(&pallet.0[*key]);
        }
        actions
    }
}

impl<K> InputActionHandler<K> for Pressable<K>
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    fn is_triggered(&self) -> bool {
        self.triggered_mapping.is_some()
    }
    
    fn clone_actions_from_pallet(&self, pallet: &ActionPallet<K>) -> Vec<InputAction> {
        if let Some(mapping_index) = self.triggered_mapping {
            let mapping_actions = &self.mappings[mapping_index].actions;
            let mut actions = Vec::new();
            for key in mapping_actions {
                actions.extend_from_slice(&pallet.0[*key]);
            }
            actions
        } else {
            Vec::new()
        }
    }
}

impl<K> InputActionHandler<K> for ClickablePong<K>
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    fn is_triggered(&self) -> bool {
        false
    }
    fn clone_actions_from_pallet(&self, _pallet: &ActionPallet<K>) -> Vec<InputAction> {
        Vec::new()
    }
}

/// Macro to process every action in the vector (for Clickable and Pressable).
macro_rules! handle_all_actions {
    ($handler:expr, $pallet:expr => {
        $($variant:ident $( ( $($binding:pat),* ) )? => $body:block),* $(,)? 
    }) => {{
        use InputAction::*;
         let handler = $handler;
         if handler.is_triggered() {
             let actions: Vec<InputAction> = handler.clone_actions_from_pallet($pallet);
             for action in actions {
                 match action {
                     $(
                         $variant $( ( $($binding),* ) )? => { $body }
                     ),*
                     _ => {}
                 }
             }
         }
    }}
}

/// Macro to apply a block to each of two optional components (Clickable and Pressable).
macro_rules! handle_triggers {
    ($clickable:expr, $pressable:expr, $pallet:expr, $handle_ident:ident => $block:block) => {{
         if let Some(mut $handle_ident) = $clickable {
             let $handle_ident = &mut *$handle_ident;
             $block
         }
         if let Some(mut $handle_ident) = $pressable {
             let $handle_ident = &mut *$handle_ident;
             $block
         }
    }}
}

/// -- Input Systems --

pub fn clickable_system<T: Send + Sync + Copy + 'static>(
    windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut clickable_q: Query<(&Aabb, &GlobalTransform, &mut Clickable<T>), Without<TextSpan>>,
    // We no longer need to set a trigger flag on ClickablePong here.
    _pong_q: Query<(&Aabb, &GlobalTransform, &ClickablePong<T>), Without<TextSpan>>,
) {
    let Some(cursor_position) = get_cursor_world_position(&windows, &camera_q) else { return };

    for (bound, transform, mut clickable) in clickable_q.iter_mut() {
        if is_cursor_within_bounds(cursor_position, transform, bound) {
            clickable.triggered = mouse_input.just_pressed(MouseButton::Left);
        }
    }
}

pub fn pressable_system<T: Copy + Send + Sync + 'static>(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Pressable<T>, &mut InteractionState)>,
) {
    for (mut pressable, mut state) in query.iter_mut() {
        // Reset the triggered mapping each frame.
        pressable.triggered_mapping = None;

        // Iterate over all mappings.
        for (i, mapping) in pressable.mappings.iter().enumerate() {
            // If any key in the mapping is just pressed, trigger this mapping.
            if mapping.allow_repeated_activation || state.0 == i {
                if mapping.keys.iter().any(|&key| keyboard_input.just_pressed(key)) {
                    pressable.triggered_mapping = Some(i);
                    state.0 = i;
                    break;
                }
            }
        }
    }
}

/// -- Trigger Systems (Audio, State Change, etc.) --
/// These systems only process the normal Clickable (and Pressable) components.
pub fn trigger_audio<K>(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K>,
        &TransientAudioPallet
    )>,
    mut audio: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    for (entity, clickable, pressable, pallet, transient_audio_pallet) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                PlaySound(key) => {
                    TransientAudioPallet::play_transient_audio(
                        entity,
                        &mut commands,
                        transient_audio_pallet,
                        key,
                        dilation.0,
                        &mut audio,
                    );
                },
            });
        });
    }
}

pub fn trigger_state_change<K>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K>
    )>,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    for (_, clickable, pressable, pallet) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
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

fn trigger_advance_dialogue<K>(
    mut query: Query<(
        Entity, 
        Option<&mut Clickable<K>>, 
        Option<&mut Pressable<K>>, 
        &ActionPallet<K>
    )>,
    mut event_writer: EventWriter<AdvanceDialogue>,
) 
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    fn send_dialogue_event(event_writer: &mut EventWriter<AdvanceDialogue>) {
        event_writer.send(AdvanceDialogue);
    }

    for (_, clickable, pressable,  pallet) in &mut query {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                AdvanceDialogue(_) => {
                    send_dialogue_event(&mut event_writer);
                }
            });
        });
    }
}

pub fn trigger_despawn<K>(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K>
    )>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    for (entity, clickable, pressable, pallet) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                Despawn => {
                    commands.entity(entity).despawn_recursive();
                }
            });
        });
    }
}

pub fn trigger_lever_state_change<K>(
    lever: Option<ResMut<Lever>>,
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K>
    )>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    if let Some(mut lever) = lever {
        let mut lever_state = lever.0;
        for (_, clickable, pressable, pallet) in query.iter_mut() {
            handle_triggers!(clickable, pressable, pallet, handle => {
                handle_all_actions!(handle, pallet => {
                    ChangeLeverState(new_lever_state) => {
                        lever_state = new_lever_state;
                    }
                });
            });
        }
        lever.0 = lever_state;
    }
}

/// This system will iterate over entities that have either a Clickable or a Pressable
/// component, and when one of their actions is Print(s), it prints s to the console.
pub fn trigger_debug_print<K>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K>
    )>,
) 
where
    K: Copy + Enum + EnumArray<Vec<InputAction>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction>>>::Array: Send + Sync,
{
    for (_, clickable, pressable, pallet) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                Print(msg) => {
                    // Print the debug message to the console.
                    println!("Print: {}", msg);
                }
            });
        });
    }
}

/// -- ClickablePong Update System --
/// This system runs on entities that have both a Clickable and a ClickablePong.
/// Instead of relying on the ClickablePong's own trigger flag, it reuses the trigger
/// from the Clickable component to update the key set (ping-pong style).
pub fn update_pong<T: Send + Sync + Copy + 'static>(
    mut query: Query<(
        &mut Clickable<T>,
        &mut ClickablePong<T>,
        &mut InteractionState,
        Option<&mut Pressable<T>>
    )>,
) {
    for (mut clickable, mut pong, mut state, pressable_opt) in query.iter_mut() {
        if pressable_opt
            .as_ref()
            .map_or(clickable.triggered, |p| p.triggered_mapping.is_some() || clickable.triggered)
        {
            match pong.direction {
                PongDirection::Forward => {
                    if state.0 >= pong.action_vector.len().saturating_sub(1) {
                        state.0 = pong.action_vector.len().saturating_sub(2);
                        pong.direction = PongDirection::Backward;
                    } else {
                        state.0 += 1;
                    }
                }
                PongDirection::Backward => {
                    if state.0 == 0 {
                        state.0 = 1;
                        pong.direction = PongDirection::Forward;
                    } else {
                        state.0 -= 1;
                    }
                }
            }
            clickable.actions = pong.action_vector[state.0].clone();
        }
    }
}

/// The ActionPallet component maps keys to vectors of InputActions.
#[derive(Component)]
pub struct ActionPallet<T>(
    pub EnumMap<T, Vec<InputAction>>
)
where
    T: Enum + EnumArray<Vec<InputAction>> + Send + Sync,
    <T as EnumArray<Vec<InputAction>>>::Array: Send + Sync;