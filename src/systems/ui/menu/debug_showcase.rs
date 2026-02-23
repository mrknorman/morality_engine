use std::collections::HashMap;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
};

use crate::{
    data::states::MainState,
    entities::{
        sprites::compound::{HollowRectangle, RectangleSides},
        text::{scaled_font_size, TextRaw},
    },
    startup::system_menu,
    systems::{
        interaction::{
            is_cursor_within_region, Clickable, Draggable, Hoverable, OptionCycler, Selectable,
            SelectableClickActivation, SelectableMenu, SelectableScopeOwner, SystemMenuActions,
        },
        ui::{
            dropdown::{self, DropdownLayerState, DropdownSurface},
            layer::UiLayer,
            menu_surface::MenuSurface,
            scroll::{
                ScrollAxis, ScrollBar, ScrollableContent, ScrollableContentExtent, ScrollableItem,
                ScrollableRoot, ScrollableViewport,
            },
            selector::SelectorSurface,
            tabs::{TabBar, TabBarState},
            window::{
                UiWindow, UiWindowContent, UiWindowContentMetrics, UiWindowOverflowPolicy,
                UiWindowTabRow, UiWindowTitle,
            },
        },
    },
};

const SHOWCASE_Z: f32 = system_menu::MENU_Z + 4.0;
const BASE_FONT_SIZE: f32 = 14.0;
const SELECTED_FONT_SIZE: f32 = 20.0;
const HOVER_FONT_SIZE: f32 = 16.0;
const WINDOW_COL_X: f32 = 360.0;
const WINDOW_ROW_Y: f32 = 186.0;

const MENU_OPTION_REGION: Vec2 = Vec2::new(320.0, 34.0);
const DROPDOWN_TRIGGER_REGION: Vec2 = Vec2::new(290.0, 36.0);
const DROPDOWN_ITEM_REGION: Vec2 = Vec2::new(264.0, 30.0);

const TABS_LABELS: [&str; 3] = ["DISPLAY", "BLOOM", "CRT"];
const TABS_CONTENT: [&str; 3] = [
    "Display primitives:\nmenu navigation, selectables,\nleft/right cycling, keyboard lock.",
    "Bloom primitives:\nhover + active tab state,\nlayer-safe interaction arbitration.",
    "CRT primitives:\ndropdown + scroll + window\ncomposition with reusable ECS parts.",
];
const DROPDOWN_VALUES: [&str; 5] = [
    "AGX",
    "REINHARD",
    "ACES FITTED",
    "TONYMCMAPFACE",
    "BLENDER FILMIC",
];

fn debug_window_color() -> Color {
    Color::srgb(0.92, 0.92, 0.92)
}

fn debug_text_color() -> Color {
    Color::srgb(0.85, 0.85, 0.85)
}

fn debug_active_text_color() -> Color {
    Color::srgb(1.0, 1.0, 1.0)
}

fn showcase_window_translation(column: usize, row: usize) -> Vec3 {
    let x = if column == 0 {
        -WINDOW_COL_X
    } else {
        WINDOW_COL_X
    };
    let y = if row == 0 {
        WINDOW_ROW_Y
    } else {
        -WINDOW_ROW_Y
    };
    Vec3::new(x, y, SHOWCASE_Z)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DebugMenuOptionKey {
    Quality,
    Refresh,
    Theme,
}

const QUALITY_VALUES: [&str; 4] = ["LOW", "MEDIUM", "HIGH", "ULTRA"];
const REFRESH_VALUES: [&str; 4] = ["30 FPS", "60 FPS", "120 FPS", "240 FPS"];
const THEME_VALUES: [&str; 3] = ["GREEN", "AMBER", "CYAN"];

fn menu_option_label(key: DebugMenuOptionKey) -> &'static str {
    match key {
        DebugMenuOptionKey::Quality => "QUALITY",
        DebugMenuOptionKey::Refresh => "REFRESH",
        DebugMenuOptionKey::Theme => "THEME",
    }
}

fn menu_option_values(key: DebugMenuOptionKey) -> &'static [&'static str] {
    match key {
        DebugMenuOptionKey::Quality => &QUALITY_VALUES,
        DebugMenuOptionKey::Refresh => &REFRESH_VALUES,
        DebugMenuOptionKey::Theme => &THEME_VALUES,
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
#[component(on_insert = DebugUiShowcaseRoot::on_insert)]
pub(super) struct DebugUiShowcaseRoot;

#[derive(Component)]
pub(super) struct DebugUiShowcaseWindow;

#[derive(Component)]
pub(super) struct DebugMenuDemoState {
    quality: usize,
    refresh: usize,
    theme: usize,
}

impl Default for DebugMenuDemoState {
    fn default() -> Self {
        Self {
            quality: 1,
            refresh: 1,
            theme: 0,
        }
    }
}

impl DebugMenuDemoState {
    fn value(&self, key: DebugMenuOptionKey) -> usize {
        match key {
            DebugMenuOptionKey::Quality => self.quality,
            DebugMenuOptionKey::Refresh => self.refresh,
            DebugMenuOptionKey::Theme => self.theme,
        }
    }

    fn value_mut(&mut self, key: DebugMenuOptionKey) -> &mut usize {
        match key {
            DebugMenuOptionKey::Quality => &mut self.quality,
            DebugMenuOptionKey::Refresh => &mut self.refresh,
            DebugMenuOptionKey::Theme => &mut self.theme,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(super) struct DebugMenuDemoOption {
    root: Entity,
    key: DebugMenuOptionKey,
    index: usize,
}

#[derive(Component, Clone, Copy)]
pub(super) struct DebugTabsDemoContent {
    window: Entity,
}

#[derive(Component, Clone, Copy)]
pub(super) struct DebugDropdownDemoState {
    selected_index: usize,
    trigger_entity: Entity,
    dropdown_parent: Entity,
}

#[derive(Component, Clone, Copy)]
pub(super) struct DebugDropdownDemoTrigger {
    owner: Entity,
}

#[derive(Component, Clone, Copy)]
pub(super) struct DebugDropdownDemoPanel {
    owner: Entity,
    size: Vec2,
}

#[derive(Component, Clone, Copy)]
pub(super) struct DebugDropdownDemoItem {
    owner: Entity,
    index: usize,
}

impl DebugUiShowcaseRoot {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        spawn_menu_selector_window(&mut world, entity);
        spawn_tabs_window(&mut world, entity);
        spawn_dropdown_window(&mut world, entity);
        spawn_scroll_window(&mut world, entity);
    }
}

fn spawn_window_base(
    world: &mut DeferredWorld,
    root: Entity,
    name: &str,
    title: &str,
    size: Vec2,
    translation: Vec3,
) -> (Entity, Entity) {
    let window_entity = world
        .commands()
        .spawn((
            Name::new(name.to_string()),
            DebugUiShowcaseWindow,
            Draggable::default(),
            UiWindow::new(
                Some(UiWindowTitle {
                    text: title.to_string(),
                    ..default()
                }),
                HollowRectangle {
                    dimensions: size,
                    thickness: 2.0,
                    color: debug_window_color(),
                    sides: RectangleSides::default(),
                },
                22.0,
                true,
                None,
            ),
            UiWindowContentMetrics::from_min_inner(size),
            UiWindowOverflowPolicy::AllowOverflow,
            Transform::from_translation(translation),
        ))
        .id();
    let content_root = world
        .commands()
        .spawn((
            Name::new(format!("{name}_content_root")),
            UiWindowContent::new(window_entity),
            Transform::default(),
        ))
        .id();
    world
        .commands()
        .entity(window_entity)
        .add_child(content_root);
    world.commands().entity(root).add_child(window_entity);
    (window_entity, content_root)
}

fn spawn_menu_selector_window(world: &mut DeferredWorld, root: Entity) {
    let (window_entity, content_root) = spawn_window_base(
        world,
        root,
        "debug_ui_showcase_menu_selector_window",
        "Menu + Selector Demo",
        Vec2::new(430.0, 270.0),
        showcase_window_translation(0, 0),
    );

    world
        .commands()
        .entity(content_root)
        .with_children(|window_parent| {
            window_parent.spawn((
                Name::new("debug_menu_demo_hint"),
                TextRaw,
                Text2d::new("Arrow Up/Down selects. Enter/Left/Right changes values."),
                TextFont {
                    font_size: scaled_font_size(12.0),
                    ..default()
                },
                TextColor(debug_text_color()),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(-194.0, 92.0, 0.2),
            ));

            let menu_root = window_parent
                .spawn((
                    Name::new("debug_menu_demo_root"),
                    DebugMenuDemoState::default(),
                    SelectableScopeOwner::new(window_entity),
                    Transform::from_xyz(-170.0, 44.0, 0.2),
                ))
                .id();
            window_parent.commands().entity(menu_root).insert(
                MenuSurface::new(menu_root)
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
            );

            window_parent
                .commands()
                .entity(menu_root)
                .with_children(|menu_parent| {
                    let option_specs = [
                        (DebugMenuOptionKey::Quality, 0, 48.0),
                        (DebugMenuOptionKey::Refresh, 1, 12.0),
                        (DebugMenuOptionKey::Theme, 2, -24.0),
                    ];

                    for (key, index, y) in option_specs {
                        menu_parent.spawn((
                            Name::new(format!("debug_menu_demo_option_{index}")),
                            TextRaw,
                            Text2d::new(""),
                            TextFont {
                                font_size: scaled_font_size(BASE_FONT_SIZE),
                                ..default()
                            },
                            TextColor(debug_text_color()),
                            Anchor::CENTER_LEFT,
                            SelectorSurface::new(menu_root, index).with_cycler(),
                            Clickable::with_region(
                                vec![SystemMenuActions::Activate],
                                MENU_OPTION_REGION,
                            ),
                            DebugMenuDemoOption {
                                root: menu_root,
                                key,
                                index,
                            },
                            Transform::from_xyz(0.0, y, 0.02),
                        ));
                    }
                });
        });
}

fn spawn_tabs_window(world: &mut DeferredWorld, root: Entity) {
    let tab_width = 120.0;
    let tab_total_width = tab_width * TABS_LABELS.len() as f32;
    let (window_entity, content_root) = spawn_window_base(
        world,
        root,
        "debug_ui_showcase_tabs_window",
        "Tabs Demo",
        Vec2::new(tab_total_width, 270.0),
        showcase_window_translation(1, 0),
    );
    world.commands().entity(window_entity).insert(
        UiWindowTabRow::from_labels(&TABS_LABELS)
            .with_tab_width(tab_width)
            .with_row_height(40.0)
            .with_color(Color::WHITE)
            .with_z(0.24),
    );

    world
        .commands()
        .entity(content_root)
        .with_children(|window_parent| {
            window_parent.spawn((
                Name::new("debug_tabs_demo_hint"),
                TextRaw,
                Text2d::new("Left/Right changes tab. Hover or click tabs directly."),
                TextFont {
                    font_size: scaled_font_size(12.0),
                    ..default()
                },
                TextColor(debug_text_color()),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(-tab_total_width * 0.5 + 12.0, 74.0, 0.2),
            ));

            window_parent.spawn((
                Name::new("debug_tabs_demo_content"),
                TextRaw,
                Text2d::new(""),
                TextFont {
                    font_size: scaled_font_size(13.0),
                    ..default()
                },
                TextColor(debug_text_color()),
                Anchor::CENTER_LEFT,
                DebugTabsDemoContent {
                    window: window_entity,
                },
                Transform::from_xyz(-tab_total_width * 0.5 + 18.0, -24.0, 0.2),
            ));
        });
}

fn spawn_dropdown_window(world: &mut DeferredWorld, root: Entity) {
    let (window_entity, content_root) = spawn_window_base(
        world,
        root,
        "debug_ui_showcase_dropdown_window",
        "Dropdown Demo",
        Vec2::new(430.0, 300.0),
        showcase_window_translation(0, 1),
    );

    world
        .commands()
        .entity(content_root)
        .with_children(|window_parent| {
            window_parent.spawn((
                Name::new("debug_dropdown_demo_hint"),
                TextRaw,
                Text2d::new("Enter/right opens, up/down navigates, enter/click commits."),
                TextFont {
                    font_size: scaled_font_size(12.0),
                    ..default()
                },
                TextColor(debug_text_color()),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(-194.0, 104.0, 0.2),
            ));

            let owner_entity = window_parent
                .spawn((
                    Name::new("debug_dropdown_demo_owner"),
                    SelectableScopeOwner::new(window_entity),
                    SelectableMenu::new(
                        0,
                        vec![],
                        vec![],
                        vec![KeyCode::Enter, KeyCode::ArrowRight],
                        false,
                    )
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
                    Transform::from_xyz(-166.0, 58.0, 0.2),
                ))
                .id();
            window_parent.commands().entity(owner_entity).insert(
                MenuSurface::new(owner_entity)
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
            );

            let mut trigger_entity = Entity::PLACEHOLDER;
            window_parent
                .commands()
                .entity(owner_entity)
                .with_children(|owner_parent| {
                    trigger_entity = owner_parent
                        .spawn((
                            Name::new("debug_dropdown_demo_trigger"),
                            TextRaw,
                            Text2d::new(""),
                            TextFont {
                                font_size: scaled_font_size(BASE_FONT_SIZE),
                                ..default()
                            },
                            TextColor(debug_text_color()),
                            Anchor::CENTER_LEFT,
                            SelectorSurface::new(owner_entity, 0),
                            Clickable::with_region(
                                vec![SystemMenuActions::Activate],
                                DROPDOWN_TRIGGER_REGION,
                            ),
                            DebugDropdownDemoTrigger {
                                owner: owner_entity,
                            },
                            Transform::from_xyz(0.0, 0.0, 0.02),
                        ))
                        .id();
                });

            let panel_size = Vec2::new(280.0, 162.0);
            let panel_entity = window_parent
                .spawn((
                    Name::new("debug_dropdown_demo_panel"),
                    DebugDropdownDemoPanel {
                        owner: owner_entity,
                        size: panel_size,
                    },
                    SelectableScopeOwner::new(window_entity),
                    DropdownSurface::new(owner_entity)
                        .with_click_activation(SelectableClickActivation::HoveredOnly),
                    Sprite::from_color(Color::BLACK, panel_size),
                    Transform::from_xyz(-12.0, -14.0, 0.24),
                    Visibility::Hidden,
                ))
                .id();
            window_parent
                .commands()
                .entity(panel_entity)
                .with_children(|panel_parent| {
                    panel_parent.spawn((
                        Name::new("debug_dropdown_demo_panel_border"),
                        HollowRectangle {
                            dimensions: panel_size - Vec2::splat(6.0),
                            thickness: 2.0,
                            color: debug_window_color(),
                            sides: RectangleSides::default(),
                        },
                        Transform::from_xyz(0.0, 0.0, 0.01),
                    ));

                    let first_y = 56.0;
                    let row_step = 30.0;
                    for (index, _) in DROPDOWN_VALUES.into_iter().enumerate() {
                        panel_parent.spawn((
                            Name::new(format!("debug_dropdown_demo_item_{index}")),
                            TextRaw,
                            Text2d::new(""),
                            TextFont {
                                font_size: scaled_font_size(BASE_FONT_SIZE),
                                ..default()
                            },
                            TextColor(debug_text_color()),
                            Anchor::CENTER_LEFT,
                            SelectorSurface::new(panel_entity, index),
                            Clickable::with_region(
                                vec![SystemMenuActions::Activate],
                                DROPDOWN_ITEM_REGION,
                            ),
                            DebugDropdownDemoItem {
                                owner: owner_entity,
                                index,
                            },
                            Transform::from_xyz(-118.0, first_y - index as f32 * row_step, 0.03),
                        ));
                    }
                });

            window_parent
                .commands()
                .entity(owner_entity)
                .insert(DebugDropdownDemoState {
                    selected_index: 0,
                    trigger_entity,
                    dropdown_parent: content_root,
                });
        });
}

fn spawn_scroll_window(world: &mut DeferredWorld, root: Entity) {
    let (window_entity, content_root) = spawn_window_base(
        world,
        root,
        "debug_ui_showcase_scroll_window",
        "Scrollable Demo",
        Vec2::new(430.0, 300.0),
        showcase_window_translation(1, 1),
    );

    world
        .commands()
        .entity(content_root)
        .with_children(|window_parent| {
            window_parent.spawn((
                Name::new("debug_scroll_demo_hint"),
                TextRaw,
                Text2d::new("Wheel, drag thumb, or hover edges to auto-scroll."),
                TextFont {
                    font_size: scaled_font_size(12.0),
                    ..default()
                },
                TextColor(debug_text_color()),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(-194.0, 104.0, 0.2),
            ));

            let viewport_size = Vec2::new(336.0, 190.0);
            let row_height = 36.0;
            let total_rows = 16usize;
            let content_extent = total_rows as f32 * row_height + 12.0;

            let scroll_root = window_parent
                .spawn((
                    Name::new("debug_scroll_demo_root"),
                    ScrollableRoot::new(window_entity, ScrollAxis::Vertical)
                        .with_edge_zones(18.0, 18.0),
                    ScrollableViewport::new(viewport_size),
                    ScrollableContentExtent(content_extent),
                    Transform::from_xyz(-22.0, -2.0, 0.2),
                ))
                .id();
            let scroll_content = window_parent
                .spawn((
                    Name::new("debug_scroll_demo_content"),
                    ScrollableContent,
                    Transform::default(),
                ))
                .id();
            window_parent
                .commands()
                .entity(scroll_root)
                .add_child(scroll_content);
            window_parent
                .commands()
                .entity(scroll_root)
                .with_children(|scroll_parent| {
                    let mut scroll_bar = ScrollBar::new(scroll_root);
                    scroll_bar.width = 10.0;
                    scroll_bar.margin = 4.0;
                    scroll_bar.track_color = debug_window_color();
                    scroll_bar.thumb_color = debug_window_color();
                    scroll_parent.spawn((
                        Name::new("debug_scroll_demo_scrollbar"),
                        scroll_bar,
                        Transform::from_xyz(0.0, 0.0, 0.22),
                    ));
                });

            window_parent
                .commands()
                .entity(scroll_content)
                .with_children(|content_parent| {
                    let first_row_center_y = viewport_size.y * 0.5 - row_height * 0.5 - 8.0;
                    for index in 0..total_rows {
                        let row_y = first_row_center_y - index as f32 * row_height;
                        content_parent.spawn((
                            Name::new(format!("debug_scroll_demo_row_{index}")),
                            ScrollableItem::new(index as u64 + 1, index, row_height),
                            HollowRectangle {
                                dimensions: Vec2::new(viewport_size.x - 32.0, row_height - 8.0),
                                thickness: 1.0,
                                color: debug_window_color(),
                                sides: RectangleSides::default(),
                            },
                            Transform::from_xyz(0.0, row_y, 0.01),
                        ));
                        content_parent.spawn((
                            Name::new(format!("debug_scroll_demo_row_text_{index}")),
                            TextRaw,
                            Text2d::new(format!("{:02}. Lorem ipsum dolor sit amet", index + 1)),
                            TextFont {
                                font_size: scaled_font_size(12.0),
                                ..default()
                            },
                            TextColor(debug_text_color()),
                            Anchor::CENTER_LEFT,
                            Transform::from_xyz(-viewport_size.x * 0.5 + 18.0, row_y, 0.03),
                        ));
                    }
                });
        });
}

fn cycle_index(current: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }
    ((current as i32 + delta).rem_euclid(len as i32)) as usize
}

pub(super) fn rebuild_debug_ui_showcase(
    commands: &mut Commands,
    showcase_root_query: &Query<Entity, With<DebugUiShowcaseRoot>>,
) {
    if let Some(existing_root) = showcase_root_query.iter().next() {
        commands.entity(existing_root).despawn_related::<Children>();
        commands.entity(existing_root).despawn();
    }

    commands.spawn((
        Name::new("debug_ui_showcase_root"),
        DebugUiShowcaseRoot,
        DespawnOnExit(MainState::Debug),
    ));
}

pub(super) fn handle_menu_demo_commands(
    mut option_query: Query<(
        &DebugMenuDemoOption,
        &mut Clickable<SystemMenuActions>,
        Option<&mut OptionCycler>,
    )>,
    mut root_query: Query<&mut DebugMenuDemoState>,
) {
    for (option, mut clickable, cycler) in option_query.iter_mut() {
        let mut delta: i32 = 0;
        if clickable.triggered {
            clickable.triggered = false;
            delta += 1;
        }

        if let Some(mut cycler) = cycler {
            if cycler.left_triggered {
                cycler.left_triggered = false;
                delta -= 1;
            }
            if cycler.right_triggered {
                cycler.right_triggered = false;
                delta += 1;
            }
        }

        if delta == 0 {
            continue;
        }

        let Ok(mut state) = root_query.get_mut(option.root) else {
            continue;
        };
        let choices = menu_option_values(option.key);
        if choices.is_empty() {
            continue;
        }
        let value = state.value_mut(option.key);
        *value = cycle_index(*value, choices.len(), delta);
    }
}

pub(super) fn sync_menu_demo_visuals(
    root_query: Query<(&SelectableMenu, &DebugMenuDemoState)>,
    mut option_query: Query<(
        &DebugMenuDemoOption,
        &Hoverable,
        &mut Text2d,
        &mut TextFont,
        &mut TextColor,
    )>,
) {
    for (option, hoverable, mut text, mut font, mut color) in option_query.iter_mut() {
        let Ok((menu, state)) = root_query.get(option.root) else {
            continue;
        };
        let values = menu_option_values(option.key);
        if values.is_empty() {
            continue;
        }
        let value_index = state.value(option.key).min(values.len() - 1);
        let selected = menu.selected_index == option.index;
        text.0 = format!(
            "{} {}  < {} >",
            if selected { ">" } else { " " },
            menu_option_label(option.key),
            values[value_index],
        );
        font.font_size = scaled_font_size(if selected {
            SELECTED_FONT_SIZE
        } else if hoverable.hovered {
            HOVER_FONT_SIZE
        } else {
            BASE_FONT_SIZE
        });
        color.0 = if selected {
            debug_active_text_color()
        } else {
            debug_text_color()
        };
    }
}

pub(super) fn sync_tabs_demo_visuals(
    tab_state_query: Query<(&TabBarState, &ChildOf), With<TabBar>>,
    mut content_query: Query<(&DebugTabsDemoContent, &mut Text2d)>,
) {
    let mut active_index_by_window: HashMap<Entity, usize> = HashMap::new();
    for (tab_state, parent) in tab_state_query.iter() {
        active_index_by_window.insert(parent.parent(), tab_state.active_index);
    }

    for (content, mut text) in content_query.iter_mut() {
        let active = active_index_by_window
            .get(&content.window)
            .copied()
            .unwrap_or(0)
            .min(TABS_CONTENT.len() - 1);
        text.0 = TABS_CONTENT[active].to_string();
    }
}

pub(super) fn handle_dropdown_trigger_commands(
    mut trigger_query: Query<(&DebugDropdownDemoTrigger, &mut Clickable<SystemMenuActions>)>,
    owner_query: Query<&DebugDropdownDemoState>,
    mut dropdown_state: ResMut<DropdownLayerState>,
    mut dropdown_query: Query<
        (Entity, &ChildOf, &UiLayer, &mut Visibility),
        With<DebugDropdownDemoPanel>,
    >,
    mut dropdown_menu_query: Query<
        &mut SelectableMenu,
        (
            With<DebugDropdownDemoPanel>,
            Without<DebugDropdownDemoTrigger>,
        ),
    >,
) {
    // Query contract:
    // - Trigger click consumption is isolated in `trigger_query`.
    // - Dropdown open/close visibility mutations are isolated in `dropdown_query`.
    // - Dropdown selectable state mutation is isolated in `dropdown_menu_query`.
    for (trigger, mut clickable) in trigger_query.iter_mut() {
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;

        let Ok(state) = owner_query.get(trigger.owner) else {
            continue;
        };

        if dropdown_state.take_suppress_toggle_once(trigger.owner) {
            continue;
        }
        if dropdown_state.is_parent_open_for_owner(trigger.owner, state.dropdown_parent) {
            dropdown::close_for_parent::<DebugDropdownDemoPanel>(
                trigger.owner,
                state.dropdown_parent,
                &mut dropdown_state,
                &mut dropdown_query,
            );
            continue;
        }
        dropdown::open_for_parent::<DebugDropdownDemoPanel>(
            trigger.owner,
            state.dropdown_parent,
            state.selected_index,
            &mut dropdown_state,
            &mut dropdown_query,
            &mut dropdown_menu_query,
        );
    }
}

pub(super) fn handle_dropdown_item_commands(
    mut item_query: Query<(
        Entity,
        &DebugDropdownDemoItem,
        &mut Clickable<SystemMenuActions>,
    )>,
    mut owner_query: Query<&mut DebugDropdownDemoState>,
    mut dropdown_state: ResMut<DropdownLayerState>,
    mut dropdown_query: Query<
        (Entity, &ChildOf, &UiLayer, &mut Visibility),
        With<DebugDropdownDemoPanel>,
    >,
) {
    // Query contract:
    // - Item click latches are consumed in `item_query`.
    // - Committed dropdown state updates are isolated in `owner_query`.
    // - Dropdown close mutations are isolated in `dropdown_query`.
    let mut chosen_by_owner: HashMap<Entity, (usize, u64)> = HashMap::new();
    for (entity, item, mut clickable) in item_query.iter_mut() {
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;
        let rank = entity.to_bits();
        match chosen_by_owner.get_mut(&item.owner) {
            Some((best_index, best_rank)) => {
                if rank > *best_rank {
                    *best_index = item.index;
                    *best_rank = rank;
                }
            }
            None => {
                chosen_by_owner.insert(item.owner, (item.index, rank));
            }
        }
    }

    for (owner, (selected, _)) in chosen_by_owner {
        let Ok(mut state) = owner_query.get_mut(owner) else {
            continue;
        };
        state.selected_index = selected.min(DROPDOWN_VALUES.len().saturating_sub(1));
        dropdown::close_for_parent::<DebugDropdownDemoPanel>(
            owner,
            state.dropdown_parent,
            &mut dropdown_state,
            &mut dropdown_query,
        );
    }
}

pub(super) fn close_dropdown_on_outside_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor: Res<crate::startup::cursor::CustomCursor>,
    mut dropdown_state: ResMut<DropdownLayerState>,
    owner_query: Query<&DebugDropdownDemoState>,
    trigger_query: Query<
        (&Transform, &GlobalTransform),
        (
            With<DebugDropdownDemoTrigger>,
            Without<DebugDropdownDemoPanel>,
        ),
    >,
    mut panel_query: Query<
        (
            &DebugDropdownDemoPanel,
            &Transform,
            &GlobalTransform,
            &mut Visibility,
        ),
        Without<DebugDropdownDemoTrigger>,
    >,
) {
    // Query contract:
    // - Trigger and panel queries are explicitly disjoint via marker filters.
    // - This prevents accidental aliasing when extending dropdown entities.
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor_position) = cursor.position else {
        return;
    };

    for (panel, panel_transform, panel_global, mut visibility) in panel_query.iter_mut() {
        if *visibility != Visibility::Visible {
            continue;
        }

        let inside_panel = is_cursor_within_region(
            cursor_position,
            panel_transform,
            panel_global,
            panel.size,
            Vec2::ZERO,
        );
        let inside_trigger = owner_query
            .get(panel.owner)
            .ok()
            .and_then(|state| trigger_query.get(state.trigger_entity).ok())
            .is_some_and(|(trigger_transform, trigger_global)| {
                is_cursor_within_region(
                    cursor_position,
                    trigger_transform,
                    trigger_global,
                    DROPDOWN_TRIGGER_REGION,
                    Vec2::ZERO,
                )
            });

        if !inside_panel && !inside_trigger {
            *visibility = Visibility::Hidden;
            dropdown_state.clear_owner(panel.owner);
        }
    }
}

pub(super) fn sync_dropdown_demo_visuals(
    dropdown_state: Res<DropdownLayerState>,
    owner_query: Query<(&SelectableMenu, &DebugDropdownDemoState)>,
    panel_menu_query: Query<&SelectableMenu, With<DebugDropdownDemoPanel>>,
    mut trigger_query: Query<
        (
            &DebugDropdownDemoTrigger,
            &Hoverable,
            &mut Text2d,
            &mut TextFont,
            &mut TextColor,
        ),
        Without<DebugDropdownDemoItem>,
    >,
    mut item_query: Query<
        (
            &DebugDropdownDemoItem,
            &Hoverable,
            &Selectable,
            &mut Text2d,
            &mut TextFont,
            &mut TextColor,
        ),
        Without<DebugDropdownDemoTrigger>,
    >,
) {
    // Query contract:
    // - Trigger/item text queries are disjoint via reciprocal `Without` filters.
    // - This keeps mutable `Text2d`/`TextFont`/`TextColor` access B0001-safe.
    for (trigger, hoverable, mut text, mut font, mut color) in trigger_query.iter_mut() {
        let Ok((menu, state)) = owner_query.get(trigger.owner) else {
            continue;
        };
        let panel_visible =
            dropdown_state.is_parent_open_for_owner(trigger.owner, state.dropdown_parent);
        let selected = menu.selected_index == 0;
        let value = DROPDOWN_VALUES[state.selected_index.min(DROPDOWN_VALUES.len() - 1)];
        text.0 = format!(
            "{} TONEMAPPER: {} {}",
            if selected { ">" } else { " " },
            value,
            if panel_visible {
                "\u{25BE}"
            } else {
                "\u{25B8}"
            },
        );
        font.font_size = scaled_font_size(if selected {
            SELECTED_FONT_SIZE
        } else if hoverable.hovered {
            HOVER_FONT_SIZE
        } else {
            BASE_FONT_SIZE
        });
        color.0 = if selected {
            debug_active_text_color()
        } else {
            debug_text_color()
        };
    }

    for (item, hoverable, selectable, mut text, mut font, mut color) in item_query.iter_mut() {
        let Ok((_, state)) = owner_query.get(item.owner) else {
            continue;
        };
        let committed = state.selected_index == item.index;
        let dropdown_selected = panel_menu_query
            .get(selectable.menu_entity)
            .is_ok_and(|menu| menu.selected_index == selectable.index);
        let label = DROPDOWN_VALUES
            .get(item.index)
            .copied()
            .unwrap_or("UNKNOWN");
        text.0 = format!("{} {}", if committed { "\u{25B8}" } else { " " }, label);
        font.font_size = scaled_font_size(if dropdown_selected {
            SELECTED_FONT_SIZE
        } else if hoverable.hovered {
            HOVER_FONT_SIZE
        } else {
            BASE_FONT_SIZE
        });
        color.0 = if dropdown_selected {
            debug_active_text_color()
        } else {
            debug_text_color()
        };
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::IntoSystem;

    use super::*;

    #[test]
    fn cycle_index_wraps_forward_and_backward() {
        assert_eq!(cycle_index(0, 4, -1), 3);
        assert_eq!(cycle_index(3, 4, 1), 0);
        assert_eq!(cycle_index(2, 4, 3), 1);
    }

    #[test]
    fn cycle_index_handles_empty_collections() {
        assert_eq!(cycle_index(0, 0, 1), 0);
        assert_eq!(cycle_index(3, 0, -1), 0);
    }

    #[test]
    fn visual_sync_systems_run_without_query_alias_panics() {
        let mut world = World::new();
        world.init_resource::<DropdownLayerState>();
        let mut schedule = Schedule::default();
        schedule.add_systems((sync_tabs_demo_visuals, sync_dropdown_demo_visuals));
        schedule.run(&mut world);
    }

    #[test]
    fn debug_showcase_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut menu_commands_system = IntoSystem::into_system(handle_menu_demo_commands);
        menu_commands_system.initialize(&mut world);

        let mut dropdown_trigger_system = IntoSystem::into_system(handle_dropdown_trigger_commands);
        dropdown_trigger_system.initialize(&mut world);

        let mut dropdown_item_system = IntoSystem::into_system(handle_dropdown_item_commands);
        dropdown_item_system.initialize(&mut world);

        let mut close_dropdown_system = IntoSystem::into_system(close_dropdown_on_outside_click);
        close_dropdown_system.initialize(&mut world);

        let mut sync_menu_visuals_system = IntoSystem::into_system(sync_menu_demo_visuals);
        sync_menu_visuals_system.initialize(&mut world);

        let mut sync_tabs_visuals_system = IntoSystem::into_system(sync_tabs_demo_visuals);
        sync_tabs_visuals_system.initialize(&mut world);

        let mut sync_dropdown_visuals_system = IntoSystem::into_system(sync_dropdown_demo_visuals);
        sync_dropdown_visuals_system.initialize(&mut world);
    }

    #[test]
    fn showcase_root_spawns_four_interactive_windows_with_primitives() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<bevy::audio::AudioSource>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();

        app.world_mut().spawn(DebugUiShowcaseRoot);
        app.update();

        let world = app.world_mut();
        let window_count = world
            .query::<(&DebugUiShowcaseWindow, &UiWindow, &Draggable)>()
            .iter(world)
            .count();
        assert_eq!(window_count, 4);

        assert!(world
            .query::<&SelectorSurface>()
            .iter(world)
            .next()
            .is_some());
        assert!(world.query::<&TabBar>().iter(world).next().is_some());
        assert!(world
            .query::<&DropdownSurface>()
            .iter(world)
            .next()
            .is_some());
        assert!(world
            .query::<&ScrollableRoot>()
            .iter(world)
            .next()
            .is_some());
    }
}
