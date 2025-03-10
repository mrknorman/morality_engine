use std::hash::Hash;
use enum_map::{
    Enum, 
    EnumArray,
    EnumMap
};
use bevy::{
    ecs::component::StorageType, 
    prelude::*, 
    render::primitives::Aabb, 
    window::PrimaryWindow
};
use crate::{
    data::{
        states::{
            DilemmaPhase, 
            GameState, 
            MainState, 
            Memory, 
            StateVector
        }, 
        stats::GameStats
    },
    systems::{
        audio::{
            AudioPlugin,
            AudioSystemsActive,
            DilatableAudio,
            TransientAudio,
            TransientAudioPallet,
        }, 
        motion::Bounce,
        time::Dilation
    }, 
    entities::{
        large_fonts::{
            AsciiActions, 
            AsciiSounds
        },
        sprites::window::{
            WindowActions,
             WindowSounds
            },
        train::{
            TrainActions,
            TrainSounds
        }
    },
    scenes::{
        dialogue::dialogue::{
            DialogueActions, 
            DialogueSounds
        }, 
        dilemma::{
            lever::{
                Lever,
                LeverState,
            },
            phases::{
                consequence::DilemmaConsequenceActions, 
                decision::{
                    DecisionActions, 
                    LeverActions
                }, 
                intro::DilemmaIntroActions, 
                results::DilemmaResultsActions
            }, 
            DilemmaSounds
        },
        ending::{
            EndingActions, 
            EndingSounds
        }, 
        loading::{
            LoadingActions, 
            LoadingSounds
        },
        menu::{
            MenuActions, 
            MenuSounds
        }
    }, 
    startup::{
        cursor::{
            CursorMode, 
            CustomCursor
        }, 
        render::MainCamera
    }
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InteractionSystem {
    Clickable,
    Pressable,
    Audio,
    AdvanceDialogue,
    LeverChange,
    Debug,
    Bounce,
    Pong,
    ResetGame,
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
    ($app:expr, $enum_type:ty, $audio_type:ty) => {
        $app.add_systems(
            Update,
            (
                Draggable::enact,
                system_entry!(clickable_system::<$enum_type>, InteractionSystem::Clickable),
                system_entry!(pressable_system::<$enum_type>, InteractionSystem::Pressable, after: InteractionSystem::Clickable),
                system_entry!(trigger_audio::<$enum_type, $audio_type>, InteractionSystem::Audio, after: InteractionSystem::Pressable),
                system_entry!(trigger_advance_dialogue::<$enum_type, $audio_type>, InteractionSystem::AdvanceDialogue, after: InteractionSystem::Audio),
                system_entry!(trigger_lever_state_change::<$enum_type, $audio_type>, InteractionSystem::LeverChange, after: InteractionSystem::AdvanceDialogue),
                system_entry!(trigger_debug_print::<$enum_type, $audio_type>, InteractionSystem::Debug, after: InteractionSystem::LeverChange),
                system_entry!(trigger_bounce::<$enum_type, $audio_type>, InteractionSystem::Bounce, after: InteractionSystem::Debug),
                system_entry!(update_pong::<$enum_type>, InteractionSystem::Pong, after: InteractionSystem::Bounce),
                system_entry!(trigger_reset_game::<$enum_type, $audio_type>, InteractionSystem::ResetGame, after: InteractionSystem::Bounce),
                system_entry!(trigger_state_change::<$enum_type, $audio_type>, InteractionSystem::StateChange, after: InteractionSystem::ResetGame),
                system_entry!(trigger_despawn::<$enum_type, $audio_type>, InteractionSystem::Despawn, after: InteractionSystem::StateChange),
            )
        );
    };
    // Fallback: if no separate audio type is provided, assume S = K.
    ($app:expr, $enum_type:ty) => {
        register_interaction_systems!($app, $enum_type, $enum_type);
    };
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<AudioPlugin>() {
            app.add_plugins(AudioPlugin);
        }
        app.add_event::<AdvanceDialogue>()
            .init_resource::<InteractionAggregate >()
            .add_systems(Startup, activate_prerequisite_states)
            .add_systems(Update, reset_clickable_aggregate.before(InteractionSystem::Clickable));

        register_interaction_systems!(app, WindowActions, WindowSounds);
        register_interaction_systems!(app, MenuActions, MenuSounds);
        register_interaction_systems!(app, LoadingActions, LoadingSounds);
        register_interaction_systems!(app, DilemmaIntroActions, DilemmaSounds);
        register_interaction_systems!(app, DilemmaConsequenceActions, DilemmaSounds);
        register_interaction_systems!(app, DilemmaResultsActions, DilemmaSounds);
        register_interaction_systems!(app, DialogueActions, DialogueSounds);
        register_interaction_systems!(app, LeverActions, DilemmaSounds);
        register_interaction_systems!(app, DecisionActions, DilemmaSounds);
        register_interaction_systems!(app, AsciiActions, AsciiSounds);
        register_interaction_systems!(app, TrainActions, TrainSounds);
        register_interaction_systems!(app, EndingActions, EndingSounds);

    }
}

fn activate_prerequisite_states(
    mut audio_state: ResMut<NextState<AudioSystemsActive>>,
) {
    audio_state.set(AudioSystemsActive::True);
}

#[derive(Event)]
pub struct AdvanceDialogue;

#[derive(Component, Default)]
pub struct IsClickable;

#[derive(Component)]
#[require(IsClickable)]
pub struct Clickable<T>
where
    T: Copy + Send + Sync,
{
    /// Keys used to look up actions in the ActionPallet.
    pub actions: Vec<T>,
    pub region : Option<Vec2>,
    pub triggered: bool,
}

pub struct KeyMapping<T>
where
    T: Copy + Send + Sync,
{
    pub keys: Vec<KeyCode>,
    pub actions: Vec<T>,
    pub allow_repeated_activation: bool,
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
    initial_state : usize,
    /// The ping–pong index and cycle state.
    direction: PongDirection,
    /// A vector of key sets (each a Vec<T>) to cycle through.
    pub action_vector: Vec<Vec<T>>,
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
                        Clickable::new(pong.action_vector[pong.initial_state].clone()),
                        InteractionState(pong.initial_state),
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
            region : None
        }
    }

    pub fn with_region(actions: Vec<T>, region : Vec2) -> Self {
        Self {
            actions,
            triggered: false,
            region : Some(region)
        }
    }
}

impl<T: Clone> ClickablePong<T> {
    pub fn new(action_vector: Vec<Vec<T>>, initial_state : usize) -> Self {
        Self {
            initial_state,
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

/// This enum now holds the “payload” type S.
/// In particular, the `PlaySound` variant carries an S.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction<S>
where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
{
    PlaySound(S),
    ChangeState(StateVector),
    AdvanceDialogue(String),
    ChangeLeverState(LeverState),
    ResetGame, 
    Bounce,
    Despawn(Option<Entity>),
    #[allow(unused)]
    Print(String),
}

/// The InputActionHandler trait is used by Clickable and Pressable
/// to look up their actions in the ActionPallet.
pub trait InputActionHandler<K, S>
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    fn is_triggered(&self) -> bool;
    fn clone_actions_from_pallet(&self, pallet: &ActionPallet<K, S>) -> Vec<InputAction<S>>;
}

impl<K, S> InputActionHandler<K, S> for Clickable<K>
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    fn is_triggered(&self) -> bool {
        self.triggered
    }
    fn clone_actions_from_pallet(&self, pallet: &ActionPallet<K, S>) -> Vec<InputAction<S>> {
        let mut actions = Vec::new();
        for key in &self.actions {
            actions.extend_from_slice(&pallet.0[*key]);
        }
        actions
    }
}

impl<K, S> InputActionHandler<K, S> for Pressable<K>
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    fn is_triggered(&self) -> bool {
        self.triggered_mapping.is_some()
    }
    fn clone_actions_from_pallet(&self, pallet: &ActionPallet<K, S>) -> Vec<InputAction<S>> {
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

impl<K, S> InputActionHandler<K, S> for ClickablePong<K>
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    fn is_triggered(&self) -> bool {
        false
    }
    fn clone_actions_from_pallet(&self, _pallet: &ActionPallet<K, S>) -> Vec<InputAction<S>> {
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
            let actions: Vec<InputAction<_>> = handler.clone_actions_from_pallet($pallet);
            for action in actions {
                match action {
                    $(
                        $variant $( ( $($binding),* ) )? => { $body }
                    ),*
                    _ => {}
                }
            }
        }
    }};
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

#[derive(Resource, Default)]
pub struct InteractionAggregate {
    option_to_click : bool,
    option_to_drag : bool,
    is_dragging : bool
}

pub fn reset_clickable_aggregate(
    mut aggregate : ResMut<InteractionAggregate>,
    mut cursor: ResMut<CustomCursor>,
) {
    if aggregate.is_dragging {
        cursor.current_mode = CursorMode::Dragging;
    } else if aggregate.option_to_click {
        cursor.current_mode = CursorMode::Clicker;
    } else if aggregate.option_to_drag {
        cursor.current_mode = CursorMode::Dragger;
    } else {
        cursor.current_mode = CursorMode::Pointer;
    }

    *aggregate = InteractionAggregate::default();
}

pub fn clickable_system<T: Send + Sync + Copy + 'static>(
        window: Single<&Window, With<PrimaryWindow>>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        mut aggregate : ResMut<InteractionAggregate>,
        camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        mut clickable_q: Query<(Option<&Aabb>, &Transform, &GlobalTransform, &mut Clickable<T>), Without<TextSpan>>,
    ) {

    let Some(cursor_position) = get_cursor_world_position(&window, &camera_q) else { return };

    for (bound, transform, global_transform, mut clickable) in clickable_q.iter_mut() {
        if let Some(region) = clickable.region {
            if is_cursor_within_region(
                cursor_position,
                &transform,
                global_transform,
                region,
                Vec2::ZERO
            ) {
                clickable.triggered = mouse_input.just_pressed(MouseButton::Left);
                aggregate.option_to_click = true;
            }
        } else if let Some(bound) = bound {
            if is_cursor_within_bounds(cursor_position, global_transform, bound) {
                clickable.triggered = mouse_input.just_pressed(MouseButton::Left);
                aggregate.option_to_click = true;
            }
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
pub fn trigger_audio<K, S>(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
        &TransientAudioPallet<S>
    )>,
    mut audio: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
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

pub fn trigger_bounce<K, S>(
    mut query: Query<(
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
        &mut Bounce
    )>
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    for (clickable, pressable, pallet, mut bounce) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                Bounce => {
                    let duration = bounce.timer.duration();
                    bounce.timer.set_elapsed(duration);
                },
            });
        });
    }
}

pub fn trigger_state_change<K, S>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
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

pub fn trigger_advance_dialogue<K, S>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
    mut event_writer: EventWriter<AdvanceDialogue>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    fn send_dialogue_event(event_writer: &mut EventWriter<AdvanceDialogue>) {
        event_writer.send(AdvanceDialogue);
    }

    for (_, clickable, pressable, pallet) in &mut query {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                AdvanceDialogue(_) => {
                    send_dialogue_event(&mut event_writer);
                }
            });
        });
    }
}

pub fn trigger_reset_game<K, S>(
    mut memory : ResMut<Memory>,
    mut stats : ResMut<GameStats>,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_sub_state: ResMut<NextState<DilemmaPhase>>,
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    for (_, clickable, pressable, pallet) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                ResetGame => {
                    *memory = Memory::default();
                    *stats = GameStats::default();
                    StateVector::new(
                        Some(MainState::Menu), 
                        Some(GameState::Loading),
                        Some(DilemmaPhase::Intro)
                    ).set_state(
                        &mut next_main_state,
                        &mut next_game_state,
                        &mut next_sub_state,
                    );
                }
            });
        });
    }
}


pub fn trigger_despawn<K, S>(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    for (entity, clickable, pressable, pallet) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                Despawn(override_entity) => {
                    commands.entity(override_entity.unwrap_or(entity)).despawn_recursive();
                }
            });
        });
    }
}

pub fn trigger_lever_state_change<K, S>(
    lever: Option<ResMut<Lever>>,
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    if let Some(mut lever) = lever {
        for (_, clickable, pressable, pallet) in query.iter_mut() {
            handle_triggers!(clickable, pressable, pallet, handle => {
                handle_all_actions!(handle, pallet => {
                    ChangeLeverState(new_lever_state) => {
                        lever.0 = new_lever_state;
                    }
                });
            });
        }
    }
}
pub fn trigger_debug_print<K, S>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    for (_, clickable, pressable, pallet) in query.iter_mut() {
        handle_triggers!(clickable, pressable, pallet, handle => {
            handle_all_actions!(handle, pallet => {
                Print(msg) => {
                    println!("Print: {}", msg);
                }
            });
        });
    }
}

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

#[derive(Component)]
pub struct ActionPallet<K, S>(
    pub EnumMap<K, Vec<InputAction<S>>>
)
where
    K: Enum + EnumArray<Vec<InputAction<S>>> + Send + Sync + Clone + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync;


pub struct DraggableRegion {
    pub region: Vec2,
    pub offset: Vec2
}

#[derive(Component)]
#[require(Transform)]
pub struct Draggable {
    pub region: Option<DraggableRegion>,
    pub offset: Vec2,
    pub dragging: bool
}

impl Default for Draggable {
    fn default() -> Self {
        Self {
            region: None,
            offset: Vec2::ZERO,
            dragging: false
        }
    }
}

impl Draggable {
    pub fn enact(
        window: Single<&Window, With<PrimaryWindow>>,
        mouse_input: Res<ButtonInput<MouseButton>>,
        camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        mut aggregate: ResMut<InteractionAggregate>,
        mut draggable_q: Query<(&GlobalTransform, &mut Draggable, &mut Transform, Option<&Aabb>), Without<TextSpan>>,
    ) {
        let Some(cursor_position) = get_cursor_world_position(&window, &camera_q) else { return };
        
        // Reset the option_to_drag flag at the beginning of the frame
        aggregate.option_to_drag = false;
        
        // First check if any entity is being actively dragged
        let any_dragging = draggable_q.iter().any(|(_, draggable, _, _)| 
            draggable.dragging && mouse_input.pressed(MouseButton::Left)
        );
        
        // Set is_dragging based on if any entity is actively being dragged
        aggregate.is_dragging = any_dragging;
        
        for (global_transform, mut draggable, mut transform, aabb) in draggable_q.iter_mut() {
            // If the user is already dragging this entity, continue the drag regardless of position
            if draggable.dragging && mouse_input.pressed(MouseButton::Left) {
                // Calculate the new position
                let new_position = cursor_position + draggable.offset;
                
                // Update the transform
                transform.translation.x = new_position.x;
                transform.translation.y = new_position.y;
                continue;
            }
            
            // Check if cursor is within bounds - handle custom region or fallback to Aabb
            let is_within_bounds = if let Some(region) = &draggable.region {
                is_cursor_within_region(
                    cursor_position, 
                    &transform, 
                    global_transform, 
                    region.region,
                    region.offset
                )
            } else if let Some(bound) = aabb {
                is_cursor_within_bounds(cursor_position, global_transform, bound)
            } else {
                // If no region or Aabb is defined, use a default small region around the transform
                let default_size = Vec2::new(10.0, 10.0);
                is_cursor_within_region(
                    cursor_position,
                    &transform,
                    global_transform,
                    default_size,
                    Vec2::ZERO
                )
            };
            
            if is_within_bounds {
                // Flag that there's something draggable under the cursor
                aggregate.option_to_drag = true;
                
                // Stop dragging if mouse button is released
                if !mouse_input.pressed(MouseButton::Left) {
                    draggable.dragging = false;
                    continue;
                }
                
                // Start dragging on mouse press
                if mouse_input.just_pressed(MouseButton::Left) {
                    draggable.dragging = true;
                    draggable.offset = global_transform.translation().truncate() - cursor_position;
                    
                    // Immediately update aggregate and cursor state
                    aggregate.is_dragging = true;
                }
            } else if draggable.dragging && !mouse_input.pressed(MouseButton::Left) {
                // If cursor is outside and mouse released, stop dragging
                draggable.dragging = false;
            }
        }
    }
}


/// Utility function for cursor handling.
pub fn get_cursor_world_position(
    window: &Single<&Window, With<PrimaryWindow>>,
    camera_q: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) -> Option<Vec2> {
    let cursor_position = window.cursor_position()?;
    let (camera, camera_transform) = camera_q.get_single().ok()?;
    let world_position = camera.viewport_to_world(camera_transform, cursor_position).ok()?;
    Some(world_position.origin.truncate())
}


pub fn is_cursor_within_bounds(cursor: Vec2, transform: &GlobalTransform, aabb: &Aabb) -> bool {
    // Get the transformation matrix
    let matrix = transform.compute_matrix();
   
    // Transform AABB corners to world space accounting for rotation
    let half_x = aabb.half_extents.x;
    let half_y = aabb.half_extents.y;
    
    // Convert Vec3A center to Vec3 
    let center = Vec3::new(aabb.center.x, aabb.center.y, aabb.center.z);
   
    // Define the four corners of the AABB in local space
    let corners = [
        Vec3::new(-half_x, -half_y, 0.0), // bottom-left
        Vec3::new(half_x, -half_y, 0.0),  // bottom-right
        Vec3::new(half_x, half_y, 0.0),   // top-right
        Vec3::new(-half_x, half_y, 0.0),  // top-left
    ];
   
    // Transform corners to world space
    let world_corners: Vec<Vec2> = corners.iter()
        .map(|corner| {
            // Apply transformation matrix including translation and rotation
            let transformed = matrix.transform_point3(*corner + center);
            Vec2::new(transformed.x, transformed.y)
        })
        .collect();
   
    // Check if cursor is inside the transformed polygon
    is_point_in_polygon(cursor, &world_corners)
}

pub fn is_cursor_within_region(
    cursor_position: Vec2,
    transform: &Transform,
    global_transform: &GlobalTransform,
    region_size: Vec2,
    region_offset: Vec2,
) -> bool {
    // Create a local transform for the region relative to the entity
    let region_local_transform = Transform::from_translation(Vec3::new(region_offset.x, region_offset.y, 0.0));
   
    // Create a matrix that transforms from local space to world space
    let model_matrix = global_transform.compute_matrix();
   
    // Calculate half size
    let half_width = region_size.x / 2.0;
    let half_height = region_size.y / 2.0;
    
    // Define the four corners of the region in local space (relative to region offset)
    let corners = [
        Vec3::new(-half_width, -half_height, 0.0), // bottom-left
        Vec3::new(half_width, -half_height, 0.0),  // bottom-right
        Vec3::new(half_width, half_height, 0.0),   // top-right
        Vec3::new(-half_width, half_height, 0.0),  // top-left
    ];
    
    // Transform corners to world space
    let world_corners: Vec<Vec2> = corners.iter()
        .map(|corner| {
            // Apply scale from transform
            let scaled_corner = Vec3::new(
                corner.x * transform.scale.x,
                corner.y * transform.scale.y,
                corner.z * transform.scale.z
            );
            
            // Apply region offset
            let offset_corner = scaled_corner + region_local_transform.translation;
            
            // Apply the full transformation matrix to get world position
            let transformed = model_matrix.transform_point3(offset_corner);
            Vec2::new(transformed.x, transformed.y)
        })
        .collect();
    
    // Check if cursor is inside the transformed polygon
    is_point_in_polygon(cursor_position, &world_corners)
}

// Helper function to check if a point is inside a polygon using the ray casting algorithm
fn is_point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    
    let mut inside = false;
    let mut j = polygon.len() - 1;
    
    for i in 0..polygon.len() {
        let vi = polygon[i];
        let vj = polygon[j];
        
        // Ray casting algorithm - count intersections
        if ((vi.y > point.y) != (vj.y > point.y)) &&
           (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
        {
            inside = !inside;
        }
        
        j = i;
    }
    
    inside
}


