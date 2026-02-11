use bevy::{ecs::query::QueryFilter, prelude::*, sprite::Anchor};
use enum_map::{Enum, EnumArray};

use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{TextButton, TextRaw},
    },
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        colors::{ColorAnchor, SYSTEM_MENU_COLOR},
        interaction::{InteractionVisualPalette, InteractionVisualState, Selectable, SelectableMenu},
    },
};

pub const PANEL_WIDTH: f32 = 420.0;
pub const BORDER_MARGIN: f32 = 18.0;
pub const BORDER_THICKNESS: f32 = 2.0;
pub const MENU_Z: f32 = 920.0;
pub const TITLE_FONT_SIZE: f32 = 24.0;
pub const HINT_FONT_SIZE: f32 = 14.0;
const OPTION_FONT_SIZE: f32 = 16.0;
const OPTION_SELECTED_FONT_SIZE: f32 = 17.5;
const OPTION_FONT_WEIGHT: FontWeight = FontWeight::NORMAL;
const OPTION_SELECTED_FONT_WEIGHT: FontWeight = FontWeight::BOLD;
const SELECTION_HIGHLIGHT_WIDTH: f32 = 290.0;
const SELECTION_HIGHLIGHT_HEIGHT: f32 = 28.0;
const SELECTION_HIGHLIGHT_Z: f32 = -0.05;
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
pub struct SystemMenuSelectionHighlight;

#[derive(Component)]
pub struct SystemMenuSelectionIndicatorsAttached;

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
                Color::BLACK,
                Color::BLACK,
                Color::BLACK,
            ),
            transform: Transform::from_xyz(0.0, y, 1.0),
            selectable: Selectable::new(menu_entity, index),
        }
    }
}

pub fn ensure_selection_indicators(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    option_query: Query<
        Entity,
        (
            With<SystemMenuOption>,
            Without<SystemMenuSelectionIndicatorsAttached>,
        ),
    >,
) {
    let triangle_mesh = meshes.add(Mesh::from(Triangle2d::new(
        Vec2::new(-SELECTION_INDICATOR_WIDTH * 0.5, SELECTION_INDICATOR_HEIGHT * 0.5),
        Vec2::new(-SELECTION_INDICATOR_WIDTH * 0.5, -SELECTION_INDICATOR_HEIGHT * 0.5),
        Vec2::new(SELECTION_INDICATOR_WIDTH * 0.5, 0.0),
    )));

    for option_entity in option_query.iter() {
        let left_material = materials.add(ColorMaterial::from(Color::BLACK));
        let right_material = materials.add(ColorMaterial::from(Color::BLACK));

        commands.entity(option_entity).with_children(|parent| {
            parent.spawn((
                Name::new("system_menu_selection_highlight"),
                SystemMenuSelectionHighlight,
                Sprite::from_color(
                    SYSTEM_MENU_COLOR,
                    Vec2::new(SELECTION_HIGHLIGHT_WIDTH, SELECTION_HIGHLIGHT_HEIGHT),
                ),
                Transform::from_xyz(0.0, 0.0, SELECTION_HIGHLIGHT_Z),
                Visibility::Hidden,
            ));

            parent.spawn((
                Name::new("system_menu_selected_indicator_left"),
                SystemMenuSelectionIndicator,
                Mesh2d(triangle_mesh.clone()),
                MeshMaterial2d(left_material),
                Transform::from_xyz(-SELECTION_INDICATOR_X, 0.0, SELECTION_INDICATOR_Z),
                Visibility::Hidden,
            ));

            parent.spawn((
                Name::new("system_menu_selected_indicator_right"),
                SystemMenuSelectionIndicator,
                Mesh2d(triangle_mesh.clone()),
                MeshMaterial2d(right_material),
                Transform::from_xyz(SELECTION_INDICATOR_X, 0.0, SELECTION_INDICATOR_Z)
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
    option_query: Query<
        (&InteractionVisualState, &InteractionVisualPalette),
        With<SystemMenuOption>,
    >,
    mut option_text_query: Query<
        (&InteractionVisualState, &mut TextFont),
        With<SystemMenuOption>,
    >,
    mut highlight_query: Query<
        (&ChildOf, &mut Visibility),
        (
            With<SystemMenuSelectionHighlight>,
            Without<SystemMenuSelectionIndicator>,
        ),
    >,
    mut indicator_query: Query<
        (&ChildOf, &mut Visibility, &MeshMaterial2d<ColorMaterial>),
        (
            With<SystemMenuSelectionIndicator>,
            Without<SystemMenuSelectionHighlight>,
        ),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (state, mut text_font) in option_text_query.iter_mut() {
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
    }

    for (parent, mut visibility) in highlight_query.iter_mut() {
        let Ok((state, _)) = option_query.get(parent.parent()) else {
            continue;
        };

        let highlighted = state.pressed || state.selected || state.hovered;
        *visibility = if highlighted {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (parent, mut visibility, material_handle) in indicator_query.iter_mut() {
        let Ok((state, palette)) = option_query.get(parent.parent()) else {
            continue;
        };

        let highlighted = state.pressed || state.selected || state.hovered;
        if highlighted {
            *visibility = Visibility::Visible;
            if let Some(material) = materials.get_mut(material_handle.0.id()) {
                material.color = palette.pressed_color;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
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

pub fn spawn_chrome(
    commands: &mut Commands,
    menu_entity: Entity,
    name_prefix: &str,
    title: &str,
    hint: &str,
    layout: SystemMenuLayout,
) {
    commands.entity(menu_entity).with_children(|parent| {
        parent.spawn((
            Name::new(format!("{name_prefix}_panel")),
            Sprite::from_color(Color::BLACK, layout.panel_size),
            Transform::from_xyz(0.0, 0.0, 0.0),
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
