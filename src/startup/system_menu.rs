use bevy::{ecs::query::QueryFilter, prelude::*, sprite::Anchor};
use enum_map::{Enum, EnumArray};

use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{TextButton, TextRaw},
    },
    systems::{
        audio::{DilatableAudio, TransientAudio, TransientAudioPallet},
        colors::{CLICKED_BUTTON, ColorAnchor, HOVERED_BUTTON, SYSTEM_MENU_COLOR},
        interaction::{InteractionVisualPalette, InteractionVisualState, Selectable, SelectableMenu},
    },
};

pub const PANEL_WIDTH: f32 = 420.0;
pub const BORDER_MARGIN: f32 = 18.0;
pub const BORDER_THICKNESS: f32 = 2.0;
pub const MENU_Z: f32 = 920.0;
pub const TITLE_FONT_SIZE: f32 = 24.0;
pub const HINT_FONT_SIZE: f32 = 14.0;

#[derive(Component)]
pub struct SystemMenu;

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
    text_button: TextButton,
    text: Text2d,
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
            text_button: TextButton,
            text: Text2d::new(label.into()),
            text_layout: TextLayout {
                justify: Justify::Center,
                ..default()
            },
            text_color: TextColor(SYSTEM_MENU_COLOR),
            color_anchor: ColorAnchor(SYSTEM_MENU_COLOR),
            visual_state: InteractionVisualState::default(),
            visual_palette: InteractionVisualPalette::new(
                SYSTEM_MENU_COLOR,
                HOVERED_BUTTON,
                CLICKED_BUTTON,
                HOVERED_BUTTON,
            ),
            transform: Transform::from_xyz(0.0, y, 1.0),
            selectable: Selectable::new(menu_entity, index),
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
