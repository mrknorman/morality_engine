use std::{cmp::Ordering, collections::HashMap, hash::Hash};
use enum_map::{
    Enum, 
    EnumArray,
    EnumMap
};
use bevy::{
    app::AppExit,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    camera::primitives::Aabb,
    window::{ClosingWindow, PrimaryWindow, WindowCloseRequested},
};
use crate::{
    data::{
        states::{
            DilemmaPhase, 
            GameState, 
            MainState, 
            PauseState,
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
        colors::{CLICKED_BUTTON, ColorAnchor, HOVERED_BUTTON},
        motion::Bounce,
        time::Dilation
    }, 
    entities::{
        large_fonts::{
            AsciiActions, 
            AsciiSounds
        },
        sprites::{
            compound::Plus,
            window::{
                Window,
                WindowActions,
                 WindowSounds
                },
        },
        train::{
            TrainActions,
            TrainSounds
        }
    },
    scenes::{
        Scene,
        SceneQueue,
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
        }
    }
};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum InteractionSystem {
    Clickable,
    Pressable,
    Selectable,
    Audio,
    AdvanceDialogue,
    LeverChange,
    #[cfg(any(debug_assertions, feature = "debug_tools"))]
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
                system_entry!(selectable_system::<$enum_type>, InteractionSystem::Selectable, after: InteractionSystem::Pressable),
                system_entry!(trigger_audio::<$enum_type, $audio_type>, InteractionSystem::Audio, after: InteractionSystem::Selectable),
                system_entry!(trigger_advance_dialogue::<$enum_type, $audio_type>, InteractionSystem::AdvanceDialogue, after: InteractionSystem::Audio),
                system_entry!(trigger_lever_state_change::<$enum_type, $audio_type>, InteractionSystem::LeverChange, after: InteractionSystem::AdvanceDialogue),
                #[cfg(any(debug_assertions, feature = "debug_tools"))]
                system_entry!(trigger_debug_print::<$enum_type, $audio_type>, InteractionSystem::Debug, after: InteractionSystem::LeverChange),
                #[cfg(any(debug_assertions, feature = "debug_tools"))]
                system_entry!(trigger_bounce::<$enum_type, $audio_type>, InteractionSystem::Bounce, after: InteractionSystem::Debug),
                #[cfg(not(any(debug_assertions, feature = "debug_tools")))]
                system_entry!(trigger_bounce::<$enum_type, $audio_type>, InteractionSystem::Bounce, after: InteractionSystem::LeverChange),
                system_entry!(update_pong::<$enum_type>, InteractionSystem::Pong, after: InteractionSystem::Bounce),
                system_entry!(trigger_reset_game::<$enum_type, $audio_type>, InteractionSystem::ResetGame, after: InteractionSystem::Bounce),
                system_entry!(trigger_exit_application::<$enum_type, $audio_type>, InteractionSystem::StateChange, after: InteractionSystem::ResetGame),
                system_entry!(trigger_state_change::<$enum_type, $audio_type>, InteractionSystem::StateChange, after: InteractionSystem::ResetGame),
                system_entry!(trigger_pause_state_change::<$enum_type, $audio_type>, InteractionSystem::StateChange, after: InteractionSystem::ResetGame),
                system_entry!(trigger_next_scene::<$enum_type, $audio_type>, InteractionSystem::StateChange, after: InteractionSystem::ResetGame),
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
        app.add_message::<AdvanceDialogue>()
            .init_resource::<InteractionAggregate >()
            .add_systems(Startup, activate_prerequisite_states)
            .add_systems(
                Update,
                (reset_clickable_aggregate, reset_interaction_visual_state)
                    .before(InteractionSystem::Clickable),
            )
            .add_systems(Update, apply_interaction_visuals.after(InteractionSystem::Selectable));

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
        register_interaction_systems!(app, OverlayMenuActions, OverlayMenuSounds);
        register_interaction_systems!(app, PauseMenuActions, PauseMenuSounds);

    }
}

fn activate_prerequisite_states(
    mut audio_state: ResMut<NextState<AudioSystemsActive>>,
) {
    audio_state.set(AudioSystemsActive::True);
}

#[derive(Message)]
pub struct AdvanceDialogue;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMenuSounds {
    Click,
    Switch,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayMenuActions {
    CloseOverlay,
    ReturnToMenu,
}

impl std::fmt::Display for OverlayMenuActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuSounds {
    Click,
    Switch,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuActions {
    Continue,
    OpenOptions,
    ExitToMenu,
    ExitToDesktop,
}

impl std::fmt::Display for PauseMenuActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionGate {
    GameplayOnly,
    PauseMenuOnly,
}

impl Default for InteractionGate {
    fn default() -> Self {
        Self::GameplayOnly
    }
}

impl InteractionGate {
    fn allows(self, paused: bool) -> bool {
        match self {
            Self::GameplayOnly => !paused,
            Self::PauseMenuOnly => paused,
        }
    }
}

pub fn interaction_gate_allows(gate: Option<&InteractionGate>, paused: bool) -> bool {
    gate.copied().unwrap_or_default().allows(paused)
}

#[derive(Copy, Clone, Component)]
pub struct ClickableCursorIcons {
    pub on_hover : CursorMode,
}

impl Default for ClickableCursorIcons{
    fn default() -> Self {
        Self{
            on_hover : CursorMode::Clicker,
        }
    }
}

#[derive(Component)]
#[require(ClickableCursorIcons, InteractionGate)]
pub struct Clickable<T>
where
    T: Copy + Send + Sync,
{
    /// Keys used to look up actions in the ActionPallet.
    pub actions: Vec<T>,
    pub region : Option<Vec2>,
    pub triggered: bool,
}


impl<T> Default for Clickable<T>
where
    T: Copy + Send + Sync,
{
    fn default() -> Self {
        Self {
            actions : vec![],
            triggered: false,
            region : None
        }
    }
}

impl<T> Clickable<T>
where
    T: Copy + Send + Sync,
{
    pub fn new(actions: Vec<T>) -> Self {
        Self {
            actions,
            ..default()
        }
    }

    pub fn with_region(actions: Vec<T>, region : Vec2) -> Self {
        Self {
            actions,
            region : Some(region),
            ..default()
        }
    }
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
#[require(InteractionState, InteractionGate)]
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

#[derive(Component, Clone)]
#[require(InteractionGate)]
pub struct SelectableMenu {
    pub selected_index: usize,
    pub up_keys: Vec<KeyCode>,
    pub down_keys: Vec<KeyCode>,
    pub activate_keys: Vec<KeyCode>,
    pub wrap: bool,
}

impl Default for SelectableMenu {
    fn default() -> Self {
        Self {
            selected_index: 0,
            up_keys: vec![KeyCode::ArrowUp],
            down_keys: vec![KeyCode::ArrowDown],
            activate_keys: vec![KeyCode::Enter],
            wrap: true,
        }
    }
}

impl SelectableMenu {
    pub fn new(
        selected_index: usize,
        up_keys: Vec<KeyCode>,
        down_keys: Vec<KeyCode>,
        activate_keys: Vec<KeyCode>,
        wrap: bool,
    ) -> Self {
        Self {
            selected_index,
            up_keys,
            down_keys,
            activate_keys,
            wrap,
        }
    }
}

#[derive(Component, Clone, Copy)]
#[require(InteractionVisualState, InteractionVisualPalette, InteractionGate)]
pub struct Selectable {
    pub menu_entity: Entity,
    pub index: usize,
}

impl Selectable {
    pub fn new(menu_entity: Entity, index: usize) -> Self {
        Self { menu_entity, index }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct InteractionVisualState {
    pub hovered: bool,
    pub pressed: bool,
    pub selected: bool,
    pub keyboard_locked: bool,
}

impl InteractionVisualState {
    pub fn clear_frame_state(&mut self) {
        self.hovered = false;
        self.pressed = false;
        self.selected = false;
        self.keyboard_locked = false;
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct InteractionVisualPalette {
    pub idle_color: Color,
    pub hovered_color: Color,
    pub pressed_color: Color,
    pub selected_color: Color,
}

impl Default for InteractionVisualPalette {
    fn default() -> Self {
        Self {
            idle_color: Color::WHITE,
            hovered_color: HOVERED_BUTTON,
            pressed_color: CLICKED_BUTTON,
            selected_color: HOVERED_BUTTON,
        }
    }
}

impl InteractionVisualPalette {
    pub fn new(
        idle_color: Color,
        hovered_color: Color,
        pressed_color: Color,
        selected_color: Color,
    ) -> Self {
        Self {
            idle_color,
            hovered_color,
            pressed_color,
            selected_color,
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

#[derive(Component, Clone)]
#[require(Clickable<T>, InteractionState)]
#[component(on_insert = ClickablePong::<T>::on_insert)]
pub struct ClickablePong<T> where
    T: Copy + Send + Sync + 'static,
{
    initial_state : usize,
    /// The ping–pong index and cycle state.
    direction: PongDirection,
    /// A vector of key sets (each a Vec<T>) to cycle through.
    pub action_vector: Vec<Vec<T>>,
}

impl<T> Default for ClickablePong<T> where
    T: Copy + Send + Sync + 'static,
{ 
    fn default() -> Self {
        Self{
            initial_state : 0,
            direction : PongDirection::Forward,
            action_vector : vec![],
        }
    }
}

impl<T: Clone> ClickablePong<T>  where
    T: Copy + Send + Sync + 'static,
{
    pub fn new(action_vector: Vec<Vec<T>>, initial_state : usize) -> Self {
        Self {
            initial_state,
            action_vector,
            ..default()
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        if let Some(pong) = world.entity(entity).get::<ClickablePong<T>>().cloned() {
            world.commands().entity(entity).insert((
                Clickable{
                    actions : pong.action_vector[pong.initial_state].clone(),
                    ..default()
                },
                InteractionState(pong.initial_state),
            ));
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
    ChangePauseState(PauseState),
    NextScene,
    AdvanceDialogue(String),
    ChangeLeverState(LeverState),
    ResetGame, 
    ExitApplication,
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
    option_to_click : Option<CursorMode>,
    option_to_drag : bool,
    is_dragging : bool
}

pub fn reset_clickable_aggregate(
    mut aggregate : ResMut<InteractionAggregate>,
    mut cursor: ResMut<CustomCursor>,
) {
    if aggregate.is_dragging {
        cursor.current_mode = CursorMode::Dragging;
    } else if let Some(mode) = aggregate.option_to_click  {
        cursor.current_mode = mode;
    } else if aggregate.option_to_drag {
        cursor.current_mode = CursorMode::Dragger;
    } else {
        cursor.current_mode = CursorMode::Pointer;
    }

    *aggregate = InteractionAggregate::default();
}

pub fn reset_interaction_visual_state(
    mut query: Query<&mut InteractionVisualState>,
) {
    for mut state in query.iter_mut() {
        state.clear_frame_state();
    }
}

pub fn apply_interaction_visuals(
    mut query: Query<(
        &InteractionVisualState,
        &InteractionVisualPalette,
        Option<&mut ColorAnchor>,
        Option<&mut TextColor>,
        Option<&mut Plus>,
    )>,
) {
    for (state, palette, color_anchor, text_color, plus) in query.iter_mut() {
        let target_color = if state.pressed {
            palette.pressed_color
        } else if state.selected {
            palette.selected_color
        } else if state.hovered {
            palette.hovered_color
        } else {
            palette.idle_color
        };

        if let Some(mut color_anchor) = color_anchor {
            color_anchor.0 = target_color;
        }
        if let Some(mut text_color) = text_color {
            text_color.0 = target_color;
        }
        if let Some(mut plus) = plus {
            plus.color = target_color;
        }
    }
}

pub fn clickable_system<T: Send + Sync + Copy + 'static>(
        mouse_input: Res<ButtonInput<MouseButton>>,
        pause_state: Option<Res<State<PauseState>>>,
        mut aggregate : ResMut<InteractionAggregate>,
        cursor : Res<CustomCursor>,
        window_query: Query<(&Window, &Transform, &GlobalTransform), Without<TextSpan>>,
        mut clickable_query: Query<
            (
                Entity,
                Option<&Aabb>,
                &Transform,
                &GlobalTransform,
                &ClickableCursorIcons,
                Option<&InteractionGate>,
                Option<&mut InteractionVisualState>,
                &mut Clickable<T>
            ),
            Without<TextSpan>
        >,
        ) {
    let paused = pause_state
        .as_ref()
        .is_some_and(|state| *state.get() == PauseState::Paused);

    // Reset click latches every frame so stale clicks cannot retrigger actions.
    for (_, _, _, _, _, _, _, mut clickable) in clickable_query.iter_mut() {
        clickable.triggered = false;
    }

    let Some(cursor_position) = cursor.position else { return };

    // Any clickable under a higher window surface is blocked, even if that
    // top window surface itself is not clickable.
    let mut top_window_z: Option<f32> = None;
    for (window, transform, global_transform) in window_query.iter() {
        let window_region = Vec2::new(
            window.boundary.dimensions.x,
            window.boundary.dimensions.y + window.header_height,
        );
        let window_offset = Vec2::new(0.0, window.header_height * 0.5);
        if is_cursor_within_region(
            cursor_position,
            transform,
            global_transform,
            window_region,
            window_offset,
        ) {
            let z = global_transform.translation().z;
            if top_window_z.is_none_or(|current| z > current) {
                top_window_z = Some(z);
            }
        }
    }

    let mut hovered_top: Option<(Entity, f32, CursorMode)> = None;

    for (entity, bound, transform, global_transform, icons, gate, _, clickable) in clickable_query.iter_mut() {
        if !interaction_gate_allows(gate, paused) {
            continue;
        }

        let is_hovered = if let Some(region) = clickable.region {
            is_cursor_within_region(
                cursor_position,
                &transform,
                global_transform,
                region,
                Vec2::ZERO,
            )
        } else if let Some(bound) = bound {
            is_cursor_within_bounds(cursor_position, global_transform, bound)
        } else {
            false
        };

        if is_hovered {
            let z = global_transform.translation().z;
            if let Some(blocking_z) = top_window_z {
                // Keep clickability for controls in the top window itself, while
                // preventing interaction with lower windows.
                if z + 0.001 < blocking_z {
                    continue;
                }
            }
            let replace = match hovered_top {
                None => true,
                Some((current_entity, current_z, _)) => {
                    z > current_z || (z == current_z && entity.index() > current_entity.index())
                }
            };
            if replace {
                hovered_top = Some((entity, z, icons.on_hover));
            }
        }
    }

    if let Some((entity, _, on_hover_mode)) = hovered_top {
        aggregate.option_to_click = Some(on_hover_mode);
        if let Ok((_, _, _, _, _, _, visual_state, _)) = clickable_query.get_mut(entity) {
            if let Some(mut visual_state) = visual_state {
                visual_state.hovered = true;
                if mouse_input.pressed(MouseButton::Left) {
                    visual_state.pressed = true;
                }
            }
        }
        if mouse_input.just_pressed(MouseButton::Left) {
            if let Ok((_, _, _, _, _, _, visual_state, mut clickable)) = clickable_query.get_mut(entity)
            {
                clickable.triggered = true;
                if let Some(mut visual_state) = visual_state {
                    visual_state.pressed = true;
                }
            }
        }
    }
}

pub fn pressable_system<T: Copy + Send + Sync + 'static>(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    mut query: Query<(
        &mut Pressable<T>,
        &mut InteractionState,
        Option<&InteractionGate>,
        Option<&mut InteractionVisualState>,
    )>,
) {
    let paused = pause_state
        .as_ref()
        .is_some_and(|state| *state.get() == PauseState::Paused);

    for (mut pressable, mut state, gate, visual_state) in query.iter_mut() {
        let mut visual_state = visual_state;
        // Reset the triggered mapping each frame.
        pressable.triggered_mapping = None;

        if !interaction_gate_allows(gate, paused) {
            continue;
        }

        // Iterate over all mappings.
        for (i, mapping) in pressable.mappings.iter().enumerate() {
            // If any key in the mapping is just pressed, trigger this mapping.
            if mapping.allow_repeated_activation || state.0 == i {
                if mapping.keys.iter().any(|&key| keyboard_input.just_pressed(key)) {
                    pressable.triggered_mapping = Some(i);
                    state.0 = i;
                    if let Some(ref mut visual_state) = visual_state {
                        visual_state.pressed = true;
                    }
                    break;
                }
            }
        }
    }
}

pub fn selectable_system<K: Copy + Send + Sync + 'static>(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<CustomCursor>,
    pause_state: Option<Res<State<PauseState>>>,
    mut menus: Query<(Entity, &mut SelectableMenu, Option<&InteractionGate>)>,
    mut menu_pointer_state: Local<HashMap<Entity, (bool, Option<Vec2>)>>,
    mut selectable_queries: ParamSet<(
        Query<
            (
                Entity,
                &Selectable,
                Option<&Aabb>,
                &Transform,
                &GlobalTransform,
                Option<&InteractionGate>,
                &Clickable<K>,
            ),
            Without<TextSpan>,
        >,
        Query<(
            &Selectable,
            &mut InteractionVisualState,
            &mut InteractionVisualPalette,
            Option<&InteractionGate>,
            &mut Clickable<K>,
        )>,
    )>,
) {
    let paused = pause_state
        .as_ref()
        .is_some_and(|state| *state.get() == PauseState::Paused);
    #[derive(Clone, Copy)]
    struct SelectableCandidate {
        entity: Entity,
        index: usize,
        z: f32,
        hovered: bool,
    }

    #[derive(Clone, Copy)]
    struct SelectionState {
        selected_index: usize,
        activate_pressed: bool,
        force_selected_click: bool,
        keyboard_locked: bool,
    }

    fn move_selection(indices: &[usize], current_index: usize, forward: bool, wrap: bool) -> usize {
        let Some(current_position) = indices.iter().position(|&index| index == current_index) else {
            return indices[0];
        };
        if forward {
            let next = current_position + 1;
            if next < indices.len() {
                indices[next]
            } else if wrap {
                indices[0]
            } else {
                indices[current_position]
            }
        } else if current_position > 0 {
            indices[current_position - 1]
        } else if wrap {
            indices[indices.len() - 1]
        } else {
            indices[current_position]
        }
    }

    let mut candidates_by_menu: HashMap<Entity, Vec<SelectableCandidate>> = HashMap::new();
    for (entity, selectable, bound, transform, global_transform, gate, clickable) in
        selectable_queries.p0().iter()
    {
        if !interaction_gate_allows(gate, paused) {
            continue;
        }

        let hovered = if let Some(cursor_position) = cursor.position {
            if let Some(region) = clickable.region {
                is_cursor_within_region(
                    cursor_position,
                    transform,
                    global_transform,
                    region,
                    Vec2::ZERO,
                )
            } else if let Some(bound) = bound {
                is_cursor_within_bounds(cursor_position, global_transform, bound)
            } else {
                false
            }
        } else {
            false
        };

        candidates_by_menu
            .entry(selectable.menu_entity)
            .or_default()
            .push(SelectableCandidate {
                entity,
                index: selectable.index,
                z: global_transform.translation().z,
                hovered,
            });
    }

    let mut selection_state_by_menu: HashMap<Entity, SelectionState> = HashMap::new();
    for (menu_entity, candidates) in candidates_by_menu.iter() {
        let Ok((_, mut menu, gate)) = menus.get_mut(*menu_entity) else {
            continue;
        };

        if !interaction_gate_allows(gate, paused) {
            continue;
        }

        if candidates.is_empty() {
            continue;
        }

        let mut indices: Vec<usize> = candidates.iter().map(|candidate| candidate.index).collect();
        indices.sort_unstable();
        indices.dedup();

        if indices.is_empty() {
            continue;
        }

        if !indices.contains(&menu.selected_index) {
            menu.selected_index = indices[0];
        }

        let pointer_state = menu_pointer_state
            .entry(*menu_entity)
            .or_insert((false, None));
        let mouse_moved = match (pointer_state.1, cursor.position) {
            (Some(prev), Some(current)) => prev.distance_squared(current) > f32::EPSILON,
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        };
        if mouse_moved {
            pointer_state.0 = false;
        }
        pointer_state.1 = cursor.position;

        let up_pressed = menu
            .up_keys
            .iter()
            .any(|&key| keyboard_input.just_pressed(key));
        let down_pressed = menu
            .down_keys
            .iter()
            .any(|&key| keyboard_input.just_pressed(key));

        if up_pressed && !down_pressed {
            menu.selected_index =
                move_selection(&indices, menu.selected_index, false, menu.wrap);
            pointer_state.0 = true;
        } else if down_pressed && !up_pressed {
            menu.selected_index =
                move_selection(&indices, menu.selected_index, true, menu.wrap);
            pointer_state.0 = true;
        } else if !pointer_state.0 {
            if let Some(top_hovered) = candidates
                .iter()
                .filter(|candidate| candidate.hovered)
                .max_by(|a, b| {
                    a.z.partial_cmp(&b.z)
                        .unwrap_or(Ordering::Equal)
                        .then_with(|| a.entity.index().cmp(&b.entity.index()))
                })
            {
                menu.selected_index = top_hovered.index;
            }
        }

        let activate_pressed = menu
            .activate_keys
            .iter()
            .any(|&key| keyboard_input.just_pressed(key));
        let force_selected_click = pointer_state.0 && mouse_input.just_pressed(MouseButton::Left);

        selection_state_by_menu.insert(
            *menu_entity,
            SelectionState {
                selected_index: menu.selected_index,
                activate_pressed,
                force_selected_click,
                keyboard_locked: pointer_state.0,
            },
        );
    }
    menu_pointer_state.retain(|entity, _| candidates_by_menu.contains_key(entity));

    for (selectable, mut visual_state, _visual_palette, gate, mut clickable) in
        selectable_queries.p1().iter_mut()
    {
        if !interaction_gate_allows(gate, paused) {
            continue;
        }

        let Some(selection_state) = selection_state_by_menu.get(&selectable.menu_entity) else {
            continue;
        };

        let is_selected = selection_state.selected_index == selectable.index;
        if is_selected && selection_state.activate_pressed {
            clickable.triggered = true;
        }
        if selection_state.force_selected_click {
            clickable.triggered = is_selected;
        }

        visual_state.selected = is_selected;
        visual_state.keyboard_locked = selection_state.keyboard_locked;
        if selection_state.keyboard_locked && !is_selected {
            visual_state.hovered = false;
        }
        if selection_state.force_selected_click {
            visual_state.pressed = is_selected;
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

pub fn trigger_next_scene<K, S>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
    mut queue : ResMut<SceneQueue>,
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
                NextScene => {
                        match queue.pop() {
                            Scene::Menu => {
                                StateVector::new(Some(MainState::Menu), None, None)
                            },
                            Scene::Loading => {
                                StateVector::new(Some(MainState::InGame), Some(GameState::Loading), None)
                            },
                            Scene::Dialogue(_) => {
                                StateVector::new(Some(MainState::InGame), Some(GameState::Dialogue), None)
                            },
                            Scene::Dilemma(_) => {
                                StateVector::new(Some(MainState::InGame), Some(GameState::Dilemma), Some(DilemmaPhase::Intro))
                            },
                            Scene::Ending(_) => {
                                StateVector::new(Some(MainState::InGame), Some(GameState::Ending), None)
                            }
                        }.set_state(
                            &mut next_main_state,
                            &mut next_game_state,
                            &mut next_sub_state,
                        );
                    }
                }
            );
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

pub fn trigger_pause_state_change<K, S>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
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
                ChangePauseState(state) => {
                    next_pause_state.set(state);
                }
            });
        });
    }
}

pub fn trigger_exit_application<K, S>(
    mut query: Query<(
        Entity,
        Option<&mut Clickable<K>>,
        Option<&mut Pressable<K>>,
        &ActionPallet<K, S>,
    )>,
    primary_window: Query<
        Entity,
        (
            With<bevy::window::Window>,
            With<PrimaryWindow>,
            Without<ClosingWindow>,
        ),
    >,
    mut close_requests: MessageWriter<WindowCloseRequested>,
    mut app_exit: MessageWriter<AppExit>,
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
                ExitApplication => {
                    if let Ok(window) = primary_window.single() {
                        close_requests.write(WindowCloseRequested { window });
                    } else {
                        app_exit.write(AppExit::Success);
                    }
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
    mut event_writer: MessageWriter<AdvanceDialogue>,
)
where
    K: Copy + Enum + EnumArray<Vec<InputAction<S>>> + Clone + Send + Sync + 'static,
    <K as EnumArray<Vec<InputAction<S>>>>::Array: Clone + Send + Sync,
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Clone + Send + Sync,
{
    fn send_dialogue_event(event_writer: &mut MessageWriter<AdvanceDialogue>) {
        event_writer.write(AdvanceDialogue);
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
    mut queue : ResMut<SceneQueue>,
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
                    *queue = SceneQueue::default();
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
                    commands.entity(override_entity.unwrap_or(entity)).despawn();
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
#[cfg(any(debug_assertions, feature = "debug_tools"))]
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

#[derive(Component, Clone, Copy, Debug)]
pub struct DraggableViewportBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl DraggableViewportBounds {
    pub fn clamp(self, position: Vec2) -> Vec2 {
        let x = if self.min.x <= self.max.x {
            position.x.clamp(self.min.x, self.max.x)
        } else {
            (self.min.x + self.max.x) * 0.5
        };
        let y = if self.min.y <= self.max.y {
            position.y.clamp(self.min.y, self.max.y)
        } else {
            (self.min.y + self.max.y) * 0.5
        };
        Vec2::new(x, y)
    }
}

#[derive(Component)]
#[require(Transform, InteractionGate)]
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
        mouse_input: Res<ButtonInput<MouseButton>>,
        cursor: Res<CustomCursor>,
        pause_state: Option<Res<State<PauseState>>>,
        mut aggregate: ResMut<InteractionAggregate>,
        mut draggable_q: Query<
            (
                Entity,
                &GlobalTransform,
                &mut Draggable,
                &mut Transform,
                Option<&Aabb>,
                Option<&DraggableViewportBounds>,
                Option<&InteractionGate>,
            ),
            Without<TextSpan>
        >,
    ) {
        let paused = pause_state
            .as_ref()
            .is_some_and(|state| *state.get() == PauseState::Paused);

        // Reset the option_to_drag flag at the beginning of the frame
        aggregate.option_to_drag = false;

        if paused {
            for (_, _, mut draggable, _, _, _, gate) in draggable_q.iter_mut() {
                if !interaction_gate_allows(gate, paused) {
                    draggable.dragging = false;
                }
            }
        }

        let Some(cursor_position) = cursor.position else {
            if !mouse_input.pressed(MouseButton::Left) {
                for (_, _, mut draggable, _, _, _, _) in draggable_q.iter_mut() {
                    draggable.dragging = false;
                }
            }
            aggregate.is_dragging = false;
            return;
        };

        let mut active_drag_target: Option<(Entity, f32)> = None;
        let mut hover_target: Option<(Entity, f32)> = None;

        for (entity, global_transform, mut draggable, transform, aabb, _, gate) in
            draggable_q.iter_mut()
        {
            if !interaction_gate_allows(gate, paused) {
                draggable.dragging = false;
                continue;
            }

            let is_within_bounds = if let Some(region) = &draggable.region {
                is_cursor_within_region(
                    cursor_position,
                    &transform,
                    global_transform,
                    region.region,
                    region.offset,
                )
            } else if let Some(bound) = aabb {
                is_cursor_within_bounds(cursor_position, global_transform, bound)
            } else {
                // If no region or Aabb is defined, use a default small region around the transform.
                let default_size = Vec2::new(10.0, 10.0);
                is_cursor_within_region(
                    cursor_position,
                    &transform,
                    global_transform,
                    default_size,
                    Vec2::ZERO,
                )
            };

            let z = global_transform.translation().z;

            if is_within_bounds {
                aggregate.option_to_drag = true;
                let replace_hover = match hover_target {
                    None => true,
                    Some((current_entity, current_z)) => {
                        z > current_z || (z == current_z && entity.index() > current_entity.index())
                    }
                };
                if replace_hover {
                    hover_target = Some((entity, z));
                }
            }

            if draggable.dragging {
                let replace_drag_target = match active_drag_target {
                    None => true,
                    Some((current_entity, current_z)) => {
                        z > current_z || (z == current_z && entity.index() > current_entity.index())
                    }
                };
                if replace_drag_target {
                    active_drag_target = Some((entity, z));
                }
            }
        }

        if !mouse_input.pressed(MouseButton::Left) {
            for (_, _, mut draggable, _, _, _, _) in draggable_q.iter_mut() {
                draggable.dragging = false;
            }
            aggregate.is_dragging = false;
            return;
        }

        if let Some((active_entity, _)) = active_drag_target {
            for (entity, _, mut draggable, mut transform, _, bounds, _) in draggable_q.iter_mut() {
                if entity == active_entity {
                    let new_position = cursor_position + draggable.offset;
                    let clamped_position = bounds
                        .map(|bounds| bounds.clamp(new_position))
                        .unwrap_or(new_position);
                    transform.translation.x = clamped_position.x;
                    transform.translation.y = clamped_position.y;
                    draggable.dragging = true;
                } else {
                    draggable.dragging = false;
                }
            }
            aggregate.is_dragging = true;
            return;
        }

        if mouse_input.just_pressed(MouseButton::Left) {
            if let Some((target_entity, _)) = hover_target {
                for (entity, global_transform, mut draggable, _, _, _, _) in draggable_q.iter_mut()
                {
                    if entity == target_entity {
                        draggable.dragging = true;
                        draggable.offset =
                            global_transform.translation().truncate() - cursor_position;
                    } else {
                        draggable.dragging = false;
                    }
                }
                aggregate.is_dragging = true;
                return;
            }
        }

        aggregate.is_dragging = false;
    }
}

pub fn is_cursor_within_bounds(cursor: Vec2, transform: &GlobalTransform, aabb: &Aabb) -> bool {
    // Get the transformation matrix
    let matrix = transform.to_matrix();
   
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
    let model_matrix = global_transform.to_matrix();
   
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

pub fn world_aabb(local: &Aabb, tf: &GlobalTransform) -> (Vec3, Vec3) {
    let he = Vec3::from(local.half_extents);
    let c = Vec3::from(local.center);
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for &sx in &[-1.0, 1.0] {
        for &sy in &[-1.0, 1.0] {
            for &sz in &[-1.0, 1.0] {
                let p = tf.transform_point(c + he * Vec3::new(sx, sy, sz));
                min = min.min(p);
                max = max.max(p);
            }
        }
    }
    (min, max)
}
