use std::collections::HashMap;

use bevy::{ecs::query::QueryFilter, prelude::*, sprite::Anchor};
use enum_map::{Enum, EnumArray};

use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, TextButton, TextRaw},
    },
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        colors::{ColorAnchor, SYSTEM_MENU_COLOR},
        interaction::{
            InteractionVisualPalette, InteractionVisualState, OptionCycler, Selectable,
            SelectableMenu,
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

#[derive(Component, Clone, Copy)]
pub struct SystemMenuSelectionIndicatorOffset(pub f32);

#[derive(Component)]
pub struct SystemMenuSelectionIndicatorsAttached;

#[derive(Component)]
pub struct SystemMenuNoSelectionIndicators;

#[derive(Component, Clone, Copy)]
pub struct SystemMenuSelectionBarStyle {
    pub offset: Vec3,
    pub size: Vec2,
    pub color: Color,
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

#[derive(Component, Clone, Copy)]
pub struct SystemMenuCycleArrowOffset(pub f32);

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
            transform: Transform::from_xyz(x, y, 1.0),
            selectable: Selectable::new(menu_entity, index),
        }
    }
}

pub fn ensure_selection_indicators(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    option_query: Query<
        (Entity, Option<&SystemMenuSelectionIndicatorOffset>),
        (
            With<SystemMenuOption>,
            Without<SystemMenuNoSelectionIndicators>,
            Without<SystemMenuSelectionIndicatorsAttached>,
        ),
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

    for (option_entity, indicator_offset) in option_query.iter() {
        let indicator_x = indicator_offset.map_or(SELECTION_INDICATOR_X, |offset| offset.0);
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
        (Entity, &SystemMenuSelectionBarStyle),
        (
            With<SystemMenuOption>,
            Without<SystemMenuSelectionBarsAttached>,
        ),
    >,
) {
    for (option_entity, style) in option_query.iter() {
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
        (&InteractionVisualState, &SystemMenuSelectionBarStyle),
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
        let up_pressed = menu
            .up_keys
            .iter()
            .any(|&key| keyboard_input.just_pressed(key));
        let down_pressed = menu
            .down_keys
            .iter()
            .any(|&key| keyboard_input.just_pressed(key));

        if (up_pressed && !down_pressed) || (down_pressed && !up_pressed) {
            TransientAudioPallet::play_transient_audio(
                menu_entity,
                commands,
                pallet,
                switch_sound,
                dilation,
                audio_query,
            );
        }
    }
}

const CYCLE_ARROW_WIDTH: f32 = 12.0;
const CYCLE_ARROW_HEIGHT: f32 = 16.0;
const CYCLE_ARROW_Z: f32 = 0.2;
// How far left/right of the value column center to place each arrow.
// The offset value passed via SystemMenuCycleArrowOffset is the center of the value column.
const CYCLE_ARROW_SPREAD: f32 = 120.0;

pub fn ensure_cycle_arrows(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    option_query: Query<
        (Entity, &SystemMenuCycleArrowOffset),
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

    for (option_entity, arrow_offset) in option_query.iter() {
        let center_x = arrow_offset.0;
        let left_material = materials.add(ColorMaterial::from(Color::BLACK));
        let right_material = materials.add(ColorMaterial::from(Color::BLACK));

        commands.entity(option_entity).with_children(|parent| {
            parent.spawn((
                Name::new("system_menu_cycle_arrow_left"),
                SystemMenuCycleArrow::Left,
                Mesh2d(left_mesh.clone()),
                MeshMaterial2d(left_material),
                Transform::from_xyz(center_x - CYCLE_ARROW_SPREAD, 0.0, CYCLE_ARROW_Z),
                Visibility::Hidden,
            ));

            parent.spawn((
                Name::new("system_menu_cycle_arrow_right"),
                SystemMenuCycleArrow::Right,
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
