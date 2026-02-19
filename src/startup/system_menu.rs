use std::collections::HashMap;

use bevy::{ecs::query::QueryFilter, prelude::*, sprite::Anchor};
use enum_map::{Enum, EnumArray};

use crate::{
    data::states::PauseState,
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, TextButton, TextRaw},
    },
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        colors::{ColorAnchor, SYSTEM_MENU_COLOR},
        interaction::{
            interaction_gate_allows_for_owner, Clickable, InteractionCapture,
            InteractionCaptureOwner, InteractionGate, InteractionVisualPalette,
            InteractionVisualState, OptionCycler, Selectable, SelectableMenu,
            SystemMenuActions,
        },
    },
};

pub const PANEL_WIDTH: f32 = 420.0;
pub const BORDER_MARGIN: f32 = 18.0;
pub const BORDER_THICKNESS: f32 = 2.0;
pub const MENU_Z: f32 = 920.0;
pub const TITLE_FONT_SIZE: f32 = scaled_font_size(24.0);
pub const HINT_FONT_SIZE: f32 = scaled_font_size(14.0);
const OPTION_FONT_SIZE: f32 = scaled_font_size(16.0);
const OPTION_SELECTED_FONT_SIZE: f32 = scaled_font_size(17.5);
const OPTION_FONT_WEIGHT: FontWeight = FontWeight::NORMAL;
const OPTION_SELECTED_FONT_WEIGHT: FontWeight = FontWeight::BOLD;
const SELECTION_INDICATOR_WIDTH: f32 = 16.0;
const SELECTION_INDICATOR_HEIGHT: f32 = 20.0;
const SELECTION_INDICATOR_X: f32 = 132.0;
const SELECTION_INDICATOR_Z: f32 = 0.2;

#[derive(Component)]
pub struct SystemMenu;

#[derive(Component)]
pub struct SystemMenuOption;

#[derive(Component)]
pub struct SystemMenuSelectionIndicator;

#[derive(Component)]
pub struct SystemMenuSelectionIndicatorsAttached;

#[derive(Component, Clone, Copy)]
pub struct SystemMenuSelectionBarStyle {
    pub offset: Vec3,
    pub size: Vec2,
    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct SystemMenuSelectionIndicatorStyle {
    pub offset_x: f32,
}

impl Default for SystemMenuSelectionIndicatorStyle {
    fn default() -> Self {
        Self {
            offset_x: SELECTION_INDICATOR_X,
        }
    }
}

#[derive(Component)]
pub struct SystemMenuSelectionBar;

#[derive(Component)]
pub struct SystemMenuSelectionBarsAttached;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SystemMenuCycleArrow {
    Left,
    Right,
}

#[derive(Component)]
pub struct SystemMenuCycleArrowsAttached;

#[derive(Clone, Copy)]
pub struct SystemMenuCycleArrowStyle {
    pub center_x: f32,
}

#[derive(Component, Clone, Copy)]
pub struct SystemMenuOptionVisualStyle {
    pub selection_indicator: Option<SystemMenuSelectionIndicatorStyle>,
    pub selection_bar: Option<SystemMenuSelectionBarStyle>,
    pub cycle_arrows: Option<SystemMenuCycleArrowStyle>,
}

impl Default for SystemMenuOptionVisualStyle {
    fn default() -> Self {
        Self {
            selection_indicator: Some(SystemMenuSelectionIndicatorStyle::default()),
            selection_bar: None,
            cycle_arrows: None,
        }
    }
}

impl SystemMenuOptionVisualStyle {
    pub fn with_indicator_offset(mut self, offset_x: f32) -> Self {
        self.selection_indicator = Some(SystemMenuSelectionIndicatorStyle { offset_x });
        self
    }

    pub fn without_selection_indicators(mut self) -> Self {
        self.selection_indicator = None;
        self
    }

    pub fn with_selection_bar(mut self, style: SystemMenuSelectionBarStyle) -> Self {
        self.selection_bar = Some(style);
        self
    }

    pub fn with_cycle_arrows(mut self, center_x: f32) -> Self {
        self.cycle_arrows = Some(SystemMenuCycleArrowStyle { center_x });
        self
    }
}

#[derive(Clone, Copy)]
pub struct SystemMenuLayout {
    pub panel_size: Vec2,
    pub title_y: f32,
    pub hint_y: f32,
}

impl SystemMenuLayout {
    pub fn new(panel_size: Vec2, title_y: f32, hint_y: f32) -> Self {
        Self {
            panel_size,
            title_y,
            hint_y,
        }
    }
}

#[derive(Bundle)]
pub struct SystemMenuOptionBundle {
    option: SystemMenuOption,
    text_button: TextButton,
    text: Text2d,
    text_font: TextFont,
    text_layout: TextLayout,
    text_color: TextColor,
    color_anchor: ColorAnchor,
    visual_state: InteractionVisualState,
    visual_palette: InteractionVisualPalette,
    visual_style: SystemMenuOptionVisualStyle,
    transform: Transform,
    selectable: Selectable,
}

impl SystemMenuOptionBundle {
    pub fn new(label: impl Into<String>, y: f32, menu_entity: Entity, index: usize) -> Self {
        Self::new_at(label, 0.0, y, menu_entity, index)
    }

    pub fn new_at(
        label: impl Into<String>,
        x: f32,
        y: f32,
        menu_entity: Entity,
        index: usize,
    ) -> Self {
        Self {
            option: SystemMenuOption,
            text_button: TextButton,
            text: Text2d::new(label.into()),
            text_font: TextFont {
                font_size: OPTION_FONT_SIZE,
                weight: OPTION_FONT_WEIGHT,
                ..default()
            },
            text_layout: TextLayout {
                justify: Justify::Center,
                ..default()
            },
            text_color: TextColor(SYSTEM_MENU_COLOR),
            color_anchor: ColorAnchor(SYSTEM_MENU_COLOR),
            visual_state: InteractionVisualState::default(),
            visual_palette: InteractionVisualPalette::new(
                SYSTEM_MENU_COLOR,
                SYSTEM_MENU_COLOR,
                SYSTEM_MENU_COLOR,
                SYSTEM_MENU_COLOR,
            ),
            visual_style: SystemMenuOptionVisualStyle::default(),
            transform: Transform::from_xyz(x, y, 1.0),
            selectable: Selectable::new(menu_entity, index),
        }
    }

    pub fn with_visual_style(mut self, style: SystemMenuOptionVisualStyle) -> Self {
        self.visual_style = style;
        self
    }
}

pub fn ensure_selection_indicators(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    option_query: Query<
        (Entity, Option<&SystemMenuOptionVisualStyle>),
        (With<SystemMenuOption>, Without<SystemMenuSelectionIndicatorsAttached>),
    >,
) {
    let triangle_mesh = meshes.add(Mesh::from(Triangle2d::new(
        Vec2::new(
            -SELECTION_INDICATOR_WIDTH * 0.5,
            SELECTION_INDICATOR_HEIGHT * 0.5,
        ),
        Vec2::new(
            -SELECTION_INDICATOR_WIDTH * 0.5,
            -SELECTION_INDICATOR_HEIGHT * 0.5,
        ),
        Vec2::new(SELECTION_INDICATOR_WIDTH * 0.5, 0.0),
    )));

    for (option_entity, style) in option_query.iter() {
        let style = style.copied().unwrap_or_default();
        let Some(indicator_style) = style.selection_indicator else {
            continue;
        };
        let indicator_x = indicator_style.offset_x;
        let left_material = materials.add(ColorMaterial::from(Color::BLACK));
        let right_material = materials.add(ColorMaterial::from(Color::BLACK));

        commands.entity(option_entity).with_children(|parent| {
            parent.spawn((
                Name::new("system_menu_selected_indicator_left"),
                SystemMenuSelectionIndicator,
                Mesh2d(triangle_mesh.clone()),
                MeshMaterial2d(left_material),
                Transform::from_xyz(-indicator_x, 0.0, SELECTION_INDICATOR_Z),
                Visibility::Hidden,
            ));

            parent.spawn((
                Name::new("system_menu_selected_indicator_right"),
                SystemMenuSelectionIndicator,
                Mesh2d(triangle_mesh.clone()),
                MeshMaterial2d(right_material),
                Transform::from_xyz(indicator_x, 0.0, SELECTION_INDICATOR_Z)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
                Visibility::Hidden,
            ));
        });

        commands
            .entity(option_entity)
            .insert(SystemMenuSelectionIndicatorsAttached);
    }
}

pub fn update_selection_indicators(
    mut option_query: Query<(Entity, &InteractionVisualState, &mut TextFont), With<SystemMenuOption>>,
    mut indicator_query: Query<
        (&ChildOf, &mut Visibility, &MeshMaterial2d<ColorMaterial>),
        With<SystemMenuSelectionIndicator>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut highlighted_by_option: HashMap<Entity, bool> = HashMap::new();

    for (entity, state, mut text_font) in option_query.iter_mut() {
        let selected = state.selected;
        text_font.font_size = if selected {
            OPTION_SELECTED_FONT_SIZE
        } else {
            OPTION_FONT_SIZE
        };
        text_font.weight = if selected {
            OPTION_SELECTED_FONT_WEIGHT
        } else {
            OPTION_FONT_WEIGHT
        };
        highlighted_by_option.insert(entity, state.pressed || state.selected || state.hovered);
    }

    for (parent, mut visibility, material_handle) in indicator_query.iter_mut() {
        let highlighted = highlighted_by_option
            .get(&parent.parent())
            .copied()
            .unwrap_or(false);
        if highlighted {
            *visibility = Visibility::Visible;
            if let Some(material) = materials.get_mut(material_handle.0.id()) {
                material.color = SYSTEM_MENU_COLOR;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn ensure_selection_bars(
    mut commands: Commands,
    option_query: Query<
        (Entity, Option<&SystemMenuOptionVisualStyle>),
        (With<SystemMenuOption>, Without<SystemMenuSelectionBarsAttached>),
    >,
) {
    for (option_entity, style) in option_query.iter() {
        let style = style.and_then(|style| style.selection_bar);
        let Some(style) = style else {
            continue;
        };
        commands.entity(option_entity).with_children(|parent| {
            parent.spawn((
                Name::new("system_menu_selection_bar"),
                SystemMenuSelectionBar,
                Sprite::from_color(style.color, style.size),
                Transform::from_translation(style.offset),
                Visibility::Hidden,
            ));
        });

        commands
            .entity(option_entity)
            .insert(SystemMenuSelectionBarsAttached);
    }
}

pub fn update_selection_bars(
    option_query: Query<
        (&InteractionVisualState, Option<&SystemMenuOptionVisualStyle>),
        With<SystemMenuOption>,
    >,
    mut bar_query: Query<
        (&ChildOf, &mut Visibility, &mut Sprite, &mut Transform),
        With<SystemMenuSelectionBar>,
    >,
) {
    for (parent, mut visibility, mut sprite, mut transform) in bar_query.iter_mut() {
        let Ok((state, style)) = option_query.get(parent.parent()) else {
            continue;
        };
        let Some(style) = style.and_then(|style| style.selection_bar) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        sprite.color = style.color;
        sprite.custom_size = Some(style.size);
        transform.translation = style.offset;

        let highlighted = state.pressed || state.selected || state.hovered;
        *visibility = if highlighted {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn click_audio_pallet<S>(
    asset_server: &Res<AssetServer>,
    click_sound: S,
) -> TransientAudioPallet<S>
where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
{
    TransientAudioPallet::new(vec![(
        click_sound,
        vec![TransientAudio::new(
            asset_server.load("./audio/effects/mech_click.ogg"),
            0.1,
            true,
            1.0,
            true,
        )],
    )])
}

pub fn switch_audio_pallet<S>(
    asset_server: &Res<AssetServer>,
    switch_sound: S,
) -> TransientAudioPallet<S>
where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
{
    TransientAudioPallet::new(vec![(
        switch_sound,
        vec![TransientAudio::new(
            asset_server.load("./audio/effects/switch.ogg"),
            0.03,
            true,
            0.2,
            false,
        )],
    )])
}

pub fn spawn_root<S, B>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    name: &str,
    translation: Vec3,
    switch_sound: S,
    extra_components: B,
) -> Entity
where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + Copy + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
    B: Bundle,
{
    commands
        .spawn((
            Name::new(name.to_string()),
            SystemMenu,
            switch_audio_pallet(asset_server, switch_sound),
            SelectableMenu::new(
                0,
                vec![KeyCode::ArrowUp],
                vec![KeyCode::ArrowDown],
                vec![KeyCode::Enter],
                true,
            ),
            Transform::from_translation(translation),
            Visibility::Visible,
            extra_components,
        ))
        .id()
}

pub fn spawn_chrome_with_marker<M>(
    commands: &mut Commands,
    menu_entity: Entity,
    name_prefix: &str,
    title: &str,
    hint: &str,
    layout: SystemMenuLayout,
    marker: M,
) where
    M: Component + Clone,
{
    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new(format!("{name_prefix}_panel")),
            Sprite::from_color(Color::BLACK, layout.panel_size),
            Transform::from_xyz(0.0, 0.0, 0.0),
            marker.clone(),
        ));
        parent.spawn((
            Name::new(format!("{name_prefix}_panel_border")),
            HollowRectangle {
                dimensions: Vec2::new(
                    layout.panel_size.x - 2.0 * BORDER_MARGIN,
                    layout.panel_size.y - 2.0 * BORDER_MARGIN,
                ),
                thickness: BORDER_THICKNESS,
                color: SYSTEM_MENU_COLOR,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.5),
            marker.clone(),
        ));
        parent.spawn((
            Name::new(format!("{name_prefix}_title")),
            TextRaw,
            Text2d::new(title),
            TextFont {
                font_size: TITLE_FONT_SIZE,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, layout.title_y, 1.0),
            marker.clone(),
        ));
        parent.spawn((
            Name::new(format!("{name_prefix}_hint")),
            TextRaw,
            Text2d::new(hint),
            TextFont {
                font_size: HINT_FONT_SIZE,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, layout.hint_y, 1.0),
            marker,
        ));
    });
}

pub fn play_navigation_sound<S, F>(
    commands: &mut Commands,
    keyboard_input: &ButtonInput<KeyCode>,
    menu_query: &Query<(Entity, &SelectableMenu, &TransientAudioPallet<S>), F>,
    audio_query: &mut Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    switch_sound: S,
    dilation: f32,
) where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + Copy + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
    F: QueryFilter,
{
    for (menu_entity, menu, pallet) in menu_query.iter() {
        play_navigation_switch(
            menu_entity,
            menu,
            pallet,
            commands,
            keyboard_input,
            audio_query,
            switch_sound,
            dilation,
        );
    }
}

pub fn play_navigation_sound_owner_scoped<S, F>(
    commands: &mut Commands,
    keyboard_input: &ButtonInput<KeyCode>,
    pause_state: Option<&Res<State<PauseState>>>,
    capture_query: &Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    menu_query: &Query<(Entity, &SelectableMenu, &TransientAudioPallet<S>, Option<&InteractionGate>), F>,
    audio_query: &mut Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    switch_sound: S,
    dilation: f32,
) where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + Copy + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
    F: QueryFilter,
{
    for (menu_entity, menu, pallet, gate) in menu_query.iter() {
        if !interaction_gate_allows_for_owner(gate, pause_state, capture_query, menu_entity) {
            continue;
        }
        play_navigation_switch(
            menu_entity,
            menu,
            pallet,
            commands,
            keyboard_input,
            audio_query,
            switch_sound,
            dilation,
        );
    }
}

pub fn navigation_switch_pressed(menu: &SelectableMenu, keyboard_input: &ButtonInput<KeyCode>) -> bool {
    let up_pressed = menu
        .up_keys
        .iter()
        .any(|&key| keyboard_input.just_pressed(key));
    let down_pressed = menu
        .down_keys
        .iter()
        .any(|&key| keyboard_input.just_pressed(key));
    (up_pressed && !down_pressed) || (down_pressed && !up_pressed)
}

pub fn play_navigation_switch<S>(
    menu_entity: Entity,
    menu: &SelectableMenu,
    pallet: &TransientAudioPallet<S>,
    commands: &mut Commands,
    keyboard_input: &ButtonInput<KeyCode>,
    audio_query: &mut Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    switch_sound: S,
    dilation: f32,
) where
    S: Enum + EnumArray<Vec<Entity>> + Send + Sync + Clone + Copy + 'static,
    <S as EnumArray<Vec<Entity>>>::Array: Send + Sync + Clone,
{
    if !navigation_switch_pressed(menu, keyboard_input) {
        return;
    }

    TransientAudioPallet::play_transient_audio(
        menu_entity,
        commands,
        pallet,
        switch_sound,
        dilation,
        audio_query,
    );
}

pub const CYCLE_ARROW_WIDTH: f32 = 12.0;
pub const CYCLE_ARROW_HALF_WIDTH: f32 = CYCLE_ARROW_WIDTH * 0.5;
const CYCLE_ARROW_HEIGHT: f32 = 16.0;
const CYCLE_ARROW_Z: f32 = 0.36;
// How far left/right of the value column center to place each arrow.
// The style center_x value is the center of the value column.
const CYCLE_ARROW_SPREAD: f32 = 120.0;

pub fn ensure_cycle_arrows(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    option_query: Query<
        (Entity, Option<&SystemMenuOptionVisualStyle>, Option<&InteractionGate>),
        (
            With<OptionCycler>,
            With<SystemMenuOption>,
            Without<SystemMenuCycleArrowsAttached>,
        ),
    >,
) {
    // Left-pointing triangle (points left)
    let left_mesh = meshes.add(Mesh::from(Triangle2d::new(
        Vec2::new(CYCLE_ARROW_WIDTH * 0.5, CYCLE_ARROW_HEIGHT * 0.5),
        Vec2::new(CYCLE_ARROW_WIDTH * 0.5, -CYCLE_ARROW_HEIGHT * 0.5),
        Vec2::new(-CYCLE_ARROW_WIDTH * 0.5, 0.0),
    )));

    // Right-pointing triangle (points right)
    let right_mesh = meshes.add(Mesh::from(Triangle2d::new(
        Vec2::new(-CYCLE_ARROW_WIDTH * 0.5, CYCLE_ARROW_HEIGHT * 0.5),
        Vec2::new(-CYCLE_ARROW_WIDTH * 0.5, -CYCLE_ARROW_HEIGHT * 0.5),
        Vec2::new(CYCLE_ARROW_WIDTH * 0.5, 0.0),
    )));

    for (option_entity, style, gate) in option_query.iter() {
        let center_x = style
            .and_then(|style| style.cycle_arrows.map(|cycle_style| cycle_style.center_x));
        let Some(center_x) = center_x else {
            continue;
        };
        let gate = gate.copied().unwrap_or_default();
        let left_material = materials.add(ColorMaterial::from(Color::BLACK));
        let right_material = materials.add(ColorMaterial::from(Color::BLACK));

        commands.entity(option_entity).with_children(|parent| {
            parent.spawn((
                Name::new("system_menu_cycle_arrow_left"),
                SystemMenuCycleArrow::Left,
                Clickable::with_region(
                    vec![SystemMenuActions::Activate],
                    Vec2::new(CYCLE_ARROW_WIDTH * 1.8, CYCLE_ARROW_HEIGHT * 1.8),
                ),
                gate,
                Mesh2d(left_mesh.clone()),
                MeshMaterial2d(left_material),
                Transform::from_xyz(center_x - CYCLE_ARROW_SPREAD, 0.0, CYCLE_ARROW_Z),
                Visibility::Hidden,
            ));

            parent.spawn((
                Name::new("system_menu_cycle_arrow_right"),
                SystemMenuCycleArrow::Right,
                Clickable::with_region(
                    vec![SystemMenuActions::Activate],
                    Vec2::new(CYCLE_ARROW_WIDTH * 1.8, CYCLE_ARROW_HEIGHT * 1.8),
                ),
                gate,
                Mesh2d(right_mesh.clone()),
                MeshMaterial2d(right_material),
                Transform::from_xyz(center_x + CYCLE_ARROW_SPREAD, 0.0, CYCLE_ARROW_Z),
                Visibility::Hidden,
            ));
        });

        commands
            .entity(option_entity)
            .insert(SystemMenuCycleArrowsAttached);
    }
}

pub fn update_cycle_arrows(
    option_query: Query<(&InteractionVisualState, &OptionCycler), With<SystemMenuOption>>,
    mut arrow_query: Query<(
        &ChildOf,
        &SystemMenuCycleArrow,
        &mut Visibility,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (parent, arrow_side, mut visibility, material_handle) in arrow_query.iter_mut() {
        let Ok((state, cycler)) = option_query.get(parent.parent()) else {
            continue;
        };

        let highlighted = state.pressed || state.selected || state.hovered;

        let arrow_allowed = match arrow_side {
            SystemMenuCycleArrow::Left => !cycler.at_min,
            SystemMenuCycleArrow::Right => !cycler.at_max,
        };

        if highlighted && arrow_allowed {
            *visibility = Visibility::Visible;
            if let Some(material) = materials.get_mut(material_handle.0.id()) {
                material.color = SYSTEM_MENU_COLOR;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn consume_cycle_arrow_clicks(
    mut arrow_query: Query<(
        &ChildOf,
        &SystemMenuCycleArrow,
        &mut Clickable<SystemMenuActions>,
    )>,
    mut option_query: Query<&mut OptionCycler, With<SystemMenuOption>>,
) {
    for (parent, side, mut clickable) in arrow_query.iter_mut() {
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;

        let Ok(mut cycler) = option_query.get_mut(parent.parent()) else {
            continue;
        };
        match side {
            SystemMenuCycleArrow::Left => {
                if !cycler.at_min {
                    cycler.left_triggered = true;
                }
            }
            SystemMenuCycleArrow::Right => {
                if !cycler.at_max {
                    cycler.right_triggered = true;
                }
            }
        }
    }
}
