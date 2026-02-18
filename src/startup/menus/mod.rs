use std::collections::{HashMap, HashSet};

use bevy::{
    app::AppExit,
    audio::Volume,
    prelude::*,
    sprite::Anchor,
    text::{TextBounds, TextLayoutInfo},
    window::{
        ClosingWindow, MonitorSelection, PresentMode, PrimaryWindow, Window, WindowCloseRequested,
        WindowMode,
    },
};

use crate::{
    data::states::{MainState, PauseState},
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, Cell, Column, TextButton, TextRaw},
    },
    startup::system_menu,
    systems::{
        audio::{continuous_audio, DilatableAudio, TransientAudio, TransientAudioPallet},
        colors::SYSTEM_MENU_COLOR,
        interaction::{
            interaction_context_active, interaction_gate_allows, option_cycler_input_system,
            Clickable, InteractionCapture, InteractionGate, InteractionSystem,
            InteractionVisualState, OptionCycler, Selectable, SelectableClickActivation,
            SelectableMenu, SystemMenuActions, SystemMenuSounds,
        },
        time::Dilation,
    },
};

mod defs;
mod dropdown;
mod selector;
mod stack;

pub use defs::{
    MenuCommand, MenuHost, MenuOptionCommand, MenuPage, MenuPageContent, MenuRoot, MenuStack,
    PauseMenuAudio,
};
use defs::*;
use dropdown::MenuDropdownState;
use selector::MenuOptionShortcut;
use stack::MenuNavigationState;

#[derive(Message, Clone, Debug)]
enum MenuIntent {
    TriggerCommand {
        menu_entity: Entity,
        command: MenuCommand,
    },
    TriggerModalButton(VideoModalButton),
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum MenuSystems {
    Core,
    Commands,
    PostCommands,
    Visual,
}

#[derive(bevy::ecs::system::SystemParam)]
struct WindowExitContext<'w, 's> {
    primary_window_queries: ParamSet<
        'w,
        's,
        (
            Query<
                'w,
                's,
                Entity,
                (With<Window>, With<PrimaryWindow>, Without<ClosingWindow>),
            >,
            Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
        ),
    >,
    close_requests: MessageWriter<'w, WindowCloseRequested>,
    app_exit: MessageWriter<'w, AppExit>,
}

fn any_resolution_dropdown_open(
    dropdown_query: &Query<&Visibility, With<VideoResolutionDropdown>>,
) -> bool {
    dropdown::any_open::<VideoResolutionDropdown>(dropdown_query)
}

fn open_dropdown_for_menu(
    menu_entity: Entity,
    selected_index: usize,
    dropdown_state: &mut MenuDropdownState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
    dropdown_menu_query: &mut Query<
        &mut SelectableMenu,
        (With<VideoResolutionDropdown>, Without<MenuRoot>),
    >,
) {
    dropdown::open_for_menu::<VideoResolutionDropdown>(
        menu_entity,
        selected_index,
        dropdown_state,
        dropdown_query,
        dropdown_menu_query,
    );
}

fn close_all_dropdowns(
    dropdown_state: &mut MenuDropdownState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
) {
    dropdown::close_all::<VideoResolutionDropdown>(dropdown_state, dropdown_query);
}

fn close_dropdowns_for_menu(
    menu_entity: Entity,
    dropdown_state: &mut MenuDropdownState,
    dropdown_query: &mut Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
) {
    dropdown::close_for_menu::<VideoResolutionDropdown>(menu_entity, dropdown_state, dropdown_query);
}

fn any_video_modal_open(modal_query: &Query<(), With<VideoModalRoot>>) -> bool {
    !modal_query.is_empty()
}

fn spawn_video_modal_base(
    commands: &mut Commands,
    menu_entity: Entity,
    name: &str,
    asset_server: &Res<AssetServer>,
    gate: InteractionGate,
    marker: impl Bundle,
) -> Entity {
    let mut modal_entity = None;
    commands.entity(menu_entity).with_children(|parent| {
        modal_entity = Some(
            parent
                .spawn((
                    Name::new(name.to_string()),
                    MenuPageContent,
                    VideoModalRoot,
                    gate,
                    system_menu::switch_audio_pallet(asset_server, SystemMenuSounds::Switch),
                    SelectableMenu::new(
                        0,
                        vec![KeyCode::ArrowLeft, KeyCode::ArrowUp],
                        vec![KeyCode::ArrowRight, KeyCode::ArrowDown],
                        vec![KeyCode::Enter],
                        true,
                    ),
                    marker,
                    Transform::from_xyz(0.0, 0.0, VIDEO_MODAL_PANEL_Z),
                ))
                .with_children(|modal| {
                    modal.spawn((
                        Name::new("video_modal_panel"),
                        Sprite::from_color(Color::BLACK, VIDEO_MODAL_PANEL_SIZE),
                        Transform::from_xyz(0.0, 0.0, 0.0),
                    ));
                    modal.spawn((
                        Name::new("video_modal_border"),
                        HollowRectangle {
                            dimensions: VIDEO_MODAL_PANEL_SIZE - Vec2::splat(14.0),
                            thickness: 2.0,
                            color: SYSTEM_MENU_COLOR,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, VIDEO_MODAL_BORDER_Z),
                    ));
                })
                .id(),
        );
    });
    modal_entity.expect("video modal entity should be spawned")
}

fn spawn_video_modal_option(
    commands: &mut Commands,
    modal_entity: Entity,
    gate: InteractionGate,
    asset_server: &Res<AssetServer>,
    button: VideoModalButton,
    index: usize,
    x: f32,
    label: &'static str,
) {
    commands.entity(modal_entity).with_children(|modal| {
        modal.spawn((
            Name::new(format!("video_modal_option_{index}")),
            MenuPageContent,
            gate,
            button,
            system_menu::SystemMenuSelectionIndicatorOffset(VIDEO_MODAL_OPTION_INDICATOR_X),
            system_menu::SystemMenuOptionBundle::new_at(label, x, VIDEO_MODAL_OPTIONS_Y, modal_entity, index),
            Clickable::with_region(vec![SystemMenuActions::Activate], VIDEO_MODAL_OPTION_REGION),
            system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
        ));
    });
}

fn spawn_apply_confirm_modal(
    commands: &mut Commands,
    menu_entity: Entity,
    asset_server: &Res<AssetServer>,
    gate: InteractionGate,
) {
    let modal_entity = spawn_video_modal_base(
        commands,
        menu_entity,
        "video_apply_confirm_modal",
        asset_server,
        gate,
        VideoApplyConfirmModal,
    );

    commands.entity(modal_entity).with_children(|modal| {
        modal.spawn((
            Name::new("video_apply_confirm_title"),
            TextRaw,
            Text2d::new("Apply these video settings?"),
            TextFont {
                font_size: scaled_font_size(20.0),
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 48.0, VIDEO_MODAL_TEXT_Z),
        ));
        modal.spawn((
            Name::new("video_apply_confirm_countdown"),
            VideoApplyCountdownText,
            TextRaw,
            Text2d::new("Reverting in 30"),
            TextFont {
                font_size: scaled_font_size(16.0),
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 8.0, VIDEO_MODAL_TEXT_Z),
        ));
    });

    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ApplyKeep,
        0,
        -VIDEO_MODAL_OPTIONS_SPREAD_X,
        "KEEP [y]",
    );
    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ApplyRevert,
        1,
        VIDEO_MODAL_OPTIONS_SPREAD_X,
        "REVERT [N/⌫]",
    );
}

fn spawn_exit_unsaved_modal(
    commands: &mut Commands,
    menu_entity: Entity,
    asset_server: &Res<AssetServer>,
    gate: InteractionGate,
) {
    let modal_entity = spawn_video_modal_base(
        commands,
        menu_entity,
        "video_unsaved_exit_modal",
        asset_server,
        gate,
        VideoExitUnsavedModal,
    );

    commands.entity(modal_entity).with_children(|modal| {
        modal.spawn((
            Name::new("video_unsaved_exit_title"),
            TextRaw,
            Text2d::new("Exit without saving changes?"),
            TextFont {
                font_size: scaled_font_size(20.0),
                weight: FontWeight::BOLD,
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, 32.0, VIDEO_MODAL_TEXT_Z),
        ));
        modal.spawn((
            Name::new("video_unsaved_exit_hint"),
            TextRaw,
            Text2d::new("Unsaved settings will be discarded."),
            TextFont {
                font_size: scaled_font_size(14.0),
                ..default()
            },
            TextColor(SYSTEM_MENU_COLOR),
            Anchor::CENTER,
            Transform::from_xyz(0.0, -2.0, VIDEO_MODAL_TEXT_Z),
        ));
    });

    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ExitWithoutSaving,
        0,
        -VIDEO_MODAL_OPTIONS_SPREAD_X,
        "EXIT [y]",
    );
    spawn_video_modal_option(
        commands,
        modal_entity,
        gate,
        asset_server,
        VideoModalButton::ExitCancel,
        1,
        VIDEO_MODAL_OPTIONS_SPREAD_X,
        "CANCEL [N/⌫]",
    );
}
fn spawn_page_content(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    menu_entity: Entity,
    page: MenuPage,
    gate: InteractionGate,
) {
    let page_def = page_definition(page);
    let is_video_page = matches!(page, MenuPage::Video);

    system_menu::spawn_chrome_with_marker(
        commands,
        menu_entity,
        page_def.name_prefix,
        page_def.title,
        page_def.hint,
        page_def.layout,
        MenuPageContent,
    );

    let click_audio = || system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click);

    if is_video_page {
        commands.entity(menu_entity).with_children(|parent| {
            parent.spawn((
                Name::new("system_video_options_table"),
                MenuPageContent,
                VideoOptionsTable,
                video_options_table(),
                Transform::from_xyz(VIDEO_TABLE_X, VIDEO_TABLE_Y, 0.95),
            ));

            let resolution_row_index = page_def
                .options
                .iter()
                .position(|option| matches!(option.command, MenuCommand::ToggleResolutionDropdown))
                .unwrap_or(VIDEO_RESOLUTION_OPTION_INDEX);
            let dropdown_rows = RESOLUTIONS.len();
            let dropdown_height = dropdown_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
            let dropdown_entity = parent
                .spawn((
                    Name::new("system_video_resolution_dropdown"),
                    MenuPageContent,
                    VideoResolutionDropdown,
                    gate,
                    system_menu::switch_audio_pallet(asset_server, SystemMenuSounds::Switch),
                    SelectableMenu::new(
                        0,
                        vec![KeyCode::ArrowUp],
                        vec![KeyCode::ArrowDown],
                        vec![KeyCode::Enter],
                        true,
                    )
                    .with_click_activation(SelectableClickActivation::HoveredOnly),
                    Sprite::from_color(
                        Color::BLACK,
                        Vec2::new(VIDEO_RESOLUTION_DROPDOWN_WIDTH, dropdown_height),
                    ),
                    Transform::from_xyz(
                        VIDEO_VALUE_COLUMN_CENTER_X,
                        dropdown_center_y_from_row_top(resolution_row_index, dropdown_rows),
                        VIDEO_RESOLUTION_DROPDOWN_Z,
                    ),
                    Visibility::Hidden,
                ))
                .id();

            parent
                .commands()
                .entity(dropdown_entity)
                .with_children(|dropdown| {
                    dropdown.spawn((
                        Name::new("system_video_resolution_dropdown_border"),
                        HollowRectangle {
                            dimensions: Vec2::new(
                                VIDEO_RESOLUTION_DROPDOWN_WIDTH - 6.0,
                                dropdown_height - 6.0,
                            ),
                            thickness: 2.0,
                            color: SYSTEM_MENU_COLOR,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, VIDEO_RESOLUTION_DROPDOWN_BORDER_Z),
                    ));

                    for (index, &(w, h)) in RESOLUTIONS.iter().enumerate() {
                        let base_y = dropdown_item_local_center_y(index, dropdown_rows);
                        dropdown
                            .spawn((
                                Name::new(format!("system_video_resolution_dropdown_item_{index}")),
                                MenuPageContent,
                                VideoResolutionDropdownItem { index },
                                TextButton,
                                gate,
                                Selectable::new(dropdown_entity, index),
                                Clickable::with_region(
                                    vec![SystemMenuActions::Activate],
                                    Vec2::new(
                                        VIDEO_RESOLUTION_DROPDOWN_WIDTH - 8.0,
                                        VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT,
                                    ),
                                ),
                                Text2d::new(format!("{}x{}", w as i32, h as i32)),
                                TextFont {
                                    font_size: VIDEO_TABLE_TEXT_SIZE,
                                    ..default()
                                },
                                TextBounds {
                                    width: Some(VIDEO_RESOLUTION_DROPDOWN_WIDTH - 8.0),
                                    height: Some(VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT),
                                },
                                TextLayout {
                                    justify: Justify::Center,
                                    ..default()
                                },
                                TextColor(SYSTEM_MENU_COLOR),
                                Anchor::CENTER,
                                Transform::from_xyz(0.0, base_y, VIDEO_RESOLUTION_DROPDOWN_ITEM_Z),
                                click_audio(),
                            ))
                            .insert(VideoResolutionDropdownItemBaseY(base_y));
                    }
                });
        });
    }

    commands.entity(menu_entity).with_children(|parent| {
        for (index, option) in page_def.options.iter().enumerate() {
            let option_label = if is_video_page { "" } else { option.label };
            let option_y = if is_video_page {
                video_row_center_y(index)
            } else {
                option.y
            };
            let option_x = if is_video_page {
                VIDEO_OPTION_SELECTOR_X
            } else {
                0.0
            };
            let clickable = if is_video_page {
                Clickable::with_region(
                    vec![SystemMenuActions::Activate],
                    Vec2::new(VIDEO_OPTION_REGION_WIDTH, VIDEO_OPTION_REGION_HEIGHT),
                )
            } else {
                Clickable::new(vec![SystemMenuActions::Activate])
            };

            if is_video_page {
                let entity_id = parent
                    .spawn((
                        Name::new(option.name),
                        MenuPageContent,
                        gate,
                        system_menu::SystemMenuNoSelectionIndicators,
                        system_menu::SystemMenuSelectionBarStyle {
                            offset: Vec3::new(
                                VIDEO_NAME_COLUMN_CENTER_X,
                                0.0,
                                VIDEO_NAME_HIGHLIGHT_Z,
                            ),
                            size: Vec2::new(
                                VIDEO_NAME_HIGHLIGHT_WIDTH,
                                VIDEO_NAME_HIGHLIGHT_HEIGHT,
                            ),
                            color: SYSTEM_MENU_COLOR,
                        },
                        VideoOptionRow { index },
                        system_menu::SystemMenuOptionBundle::new_at(
                            option_label,
                            option_x,
                            option_y,
                            menu_entity,
                            index,
                        ),
                        clickable,
                        MenuOptionCommand(option.command.clone()),
                        click_audio(),
                    ))
                    .id();

                if let Some(shortcut) = option.shortcut {
                    parent
                        .commands()
                        .entity(entity_id)
                        .insert(MenuOptionShortcut(shortcut));
                }

                if option.cyclable {
                    parent.commands().entity(entity_id).insert((
                        OptionCycler::default(),
                        system_menu::SystemMenuCycleArrowOffset(VIDEO_VALUE_COLUMN_CENTER_X),
                    ));
                }
            } else {
                let entity_id = parent
                    .spawn((
                    Name::new(option.name),
                    MenuPageContent,
                    gate,
                    system_menu::SystemMenuOptionBundle::new_at(
                        option_label,
                        option_x,
                        option_y,
                        menu_entity,
                        index,
                    ),
                    clickable,
                    MenuOptionCommand(option.command.clone()),
                    click_audio(),
                ))
                    .id();

                if let Some(shortcut) = option.shortcut {
                    parent
                        .commands()
                        .entity(entity_id)
                        .insert(MenuOptionShortcut(shortcut));
                }
            }
        }
    });
}

fn clear_page_content(
    commands: &mut Commands,
    menu_entity: Entity,
    page_content_query: &Query<(Entity, &ChildOf), With<MenuPageContent>>,
) {
    let content_entities: Vec<Entity> = page_content_query
        .iter()
        .filter_map(|(entity, parent)| {
            if parent.parent() == menu_entity {
                Some(entity)
            } else {
                None
            }
        })
        .collect();

    for entity in content_entities {
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).despawn();
    }
}

fn rebuild_menu_page(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    menu_entity: Entity,
    page: MenuPage,
    gate: InteractionGate,
    page_content_query: &Query<(Entity, &ChildOf), With<MenuPageContent>>,
) {
    clear_page_content(commands, menu_entity, page_content_query);
    spawn_page_content(commands, asset_server, menu_entity, page, gate);
}

fn request_application_exit(
    primary_window: &Query<
        Entity,
        (
            With<bevy::window::Window>,
            With<PrimaryWindow>,
            Without<ClosingWindow>,
        ),
    >,
    close_requests: &mut MessageWriter<WindowCloseRequested>,
    app_exit: &mut MessageWriter<AppExit>,
) {
    if let Ok(window) = primary_window.single() {
        close_requests.write(WindowCloseRequested { window });
    } else {
        app_exit.write(AppExit::Success);
    }
}

fn resolution_index_from_window(window: &Window) -> usize {
    let current = (window.resolution.width(), window.resolution.height());
    RESOLUTIONS
        .iter()
        .position(|&(w, h)| (current.0 - w).abs() < 1.0 && (current.1 - h).abs() < 1.0)
        .unwrap_or(0)
}

fn apply_resolution(window: &mut Window, index: usize) {
    let clamped = index.min(RESOLUTIONS.len() - 1);
    let (w, h) = RESOLUTIONS[clamped];
    window.resolution.set(w, h);
}

fn snapshot_from_window(window: &Window) -> VideoSettingsSnapshot {
    let display_mode = match window.mode {
        WindowMode::Windowed => VideoDisplayMode::Windowed,
        _ => VideoDisplayMode::Borderless,
    };
    let vsync_enabled = matches!(window.present_mode, PresentMode::AutoVsync);
    VideoSettingsSnapshot {
        display_mode,
        resolution_index: resolution_index_from_window(window),
        vsync_enabled,
    }
}

fn apply_snapshot_to_window(window: &mut Window, snapshot: VideoSettingsSnapshot) {
    window.mode = match snapshot.display_mode {
        VideoDisplayMode::Windowed => WindowMode::Windowed,
        VideoDisplayMode::Borderless => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
    };
    apply_resolution(window, snapshot.resolution_index);
    window.present_mode = if snapshot.vsync_enabled {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
}

fn sync_video_table_values(
    settings: Res<VideoSettingsState>,
    video_option_query: Query<(&VideoOptionRow, &InteractionVisualState)>,
    table_query: Query<&Children, With<VideoOptionsTable>>,
    column_children_query: Query<&Children, With<Column>>,
    cell_children_query: Query<&Children, With<Cell>>,
    mut text_query: Query<(&mut Text2d, &mut TextColor, &mut TextFont, &mut Transform)>,
) {
    if !settings.initialized {
        return;
    }

    let mut selected_by_row: HashMap<usize, bool> = HashMap::new();
    let mut highlighted_by_row: HashMap<usize, bool> = HashMap::new();
    for (row, state) in video_option_query.iter() {
        selected_by_row.insert(row.index, state.selected);
        highlighted_by_row.insert(row.index, state.selected || state.hovered || state.pressed);
    }

    let value_strings = [
        display_mode_text(settings.pending.display_mode).to_string(),
        format!(
            "{}x{}",
            RESOLUTIONS[settings.pending.resolution_index.min(RESOLUTIONS.len() - 1)].0 as i32,
            RESOLUTIONS[settings.pending.resolution_index.min(RESOLUTIONS.len() - 1)].1 as i32,
        ),
        present_mode_text(settings.pending.vsync_enabled).to_string(),
        String::new(),
        String::new(),
        String::new(),
    ];

    for table_children in table_query.iter() {
        if table_children.len() < 2 {
            continue;
        }

        for (column_index, column_entity) in table_children.iter().enumerate() {
            let Ok(cells) = column_children_query.get(column_entity) else {
                continue;
            };

            for (row_index, cell_entity) in cells.iter().enumerate() {
                let Ok(cell_children) = cell_children_query.get(cell_entity) else {
                    continue;
                };

                for child in cell_children.iter() {
                    let Ok((mut text, mut color, mut font, mut transform)) =
                        text_query.get_mut(child)
                    else {
                        continue;
                    };

                    if column_index == 1 {
                        let Some(value) = value_strings.get(row_index) else {
                            break;
                        };
                        if text.0 != *value {
                            text.0 = value.clone();
                        }
                    }

                    let selected = selected_by_row.get(&row_index).copied().unwrap_or(false);
                    if selected {
                        font.font_size = VIDEO_TABLE_TEXT_SELECTED_SIZE;
                        font.weight = FontWeight::BOLD;
                    } else {
                        font.font_size = VIDEO_TABLE_TEXT_SIZE;
                        font.weight = FontWeight::NORMAL;
                    }

                    if column_index == 0
                        && highlighted_by_row.get(&row_index).copied().unwrap_or(false)
                    {
                        color.0 = Color::srgb(0.0, 0.08, 0.0);
                    } else {
                        color.0 = SYSTEM_MENU_COLOR;
                    }

                    transform.translation.z = VIDEO_TABLE_TEXT_Z;
                    break;
                }
            }
        }
    }
}

fn initialize_video_settings_from_window(
    mut settings: ResMut<VideoSettingsState>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if settings.initialized {
        return;
    }
    let Ok(window) = window_query.single() else {
        return;
    };
    let snapshot = snapshot_from_window(window);
    settings.saved = snapshot;
    settings.pending = snapshot;
    settings.initialized = true;
}

fn sync_resolution_dropdown_items(
    settings: Res<VideoSettingsState>,
    dropdown_state: Res<MenuDropdownState>,
    mut item_query: Query<(
        &VideoResolutionDropdownItem,
        &InteractionVisualState,
        &mut Text2d,
        &mut TextColor,
        &mut TextFont,
    )>,
) {
    if !settings.initialized {
        return;
    }

    let selected_index = settings.pending.resolution_index.min(RESOLUTIONS.len() - 1);
    let dropdown_open = dropdown_state.open_menu.is_some();

    for (item, state, mut text, mut color, mut font) in item_query.iter_mut() {
        let (w, h) = RESOLUTIONS[item.index.min(RESOLUTIONS.len() - 1)];
        text.0 = format!("{}x{}", w as i32, h as i32);

        let focused = dropdown_open && (state.selected || state.hovered || state.pressed);
        if focused {
            color.0 = SYSTEM_MENU_COLOR;
            font.font_size = VIDEO_TABLE_TEXT_SELECTED_SIZE;
            font.weight = FontWeight::BOLD;
        } else if item.index == selected_index {
            color.0 = SYSTEM_MENU_COLOR;
            font.font_size = VIDEO_TABLE_TEXT_SIZE;
            font.weight = FontWeight::BOLD;
        } else {
            color.0 = SYSTEM_MENU_COLOR;
            font.font_size = VIDEO_TABLE_TEXT_SIZE;
            font.weight = FontWeight::NORMAL;
        }
    }
}

fn ensure_resolution_dropdown_value_arrows(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    dropdown_query: Query<
        Entity,
        (
            With<VideoResolutionDropdown>,
            Without<VideoResolutionDropdownValueArrowAttached>,
        ),
    >,
) {
    let triangle_mesh = meshes.add(Mesh::from(Triangle2d::new(
        Vec2::new(
            -VIDEO_RESOLUTION_DROPDOWN_ARROW_WIDTH * 0.5,
            VIDEO_RESOLUTION_DROPDOWN_ARROW_HEIGHT * 0.5,
        ),
        Vec2::new(
            -VIDEO_RESOLUTION_DROPDOWN_ARROW_WIDTH * 0.5,
            -VIDEO_RESOLUTION_DROPDOWN_ARROW_HEIGHT * 0.5,
        ),
        Vec2::new(VIDEO_RESOLUTION_DROPDOWN_ARROW_WIDTH * 0.5, 0.0),
    )));

    for dropdown_entity in dropdown_query.iter() {
        let material = materials.add(ColorMaterial::from(SYSTEM_MENU_COLOR));
        commands.entity(dropdown_entity).with_children(|parent| {
            parent.spawn((
                Name::new("system_video_resolution_dropdown_value_arrow_left"),
                VideoResolutionDropdownValueArrow,
                VideoResolutionDropdownValueArrowSide::Left,
                Mesh2d(triangle_mesh.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_xyz(
                    -VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD,
                    dropdown_item_local_center_y(0, RESOLUTIONS.len()),
                    VIDEO_RESOLUTION_DROPDOWN_ARROW_Z,
                ),
                Visibility::Hidden,
            ));

            parent.spawn((
                Name::new("system_video_resolution_dropdown_value_arrow_right"),
                VideoResolutionDropdownValueArrow,
                VideoResolutionDropdownValueArrowSide::Right,
                Mesh2d(triangle_mesh.clone()),
                MeshMaterial2d(material),
                Transform::from_xyz(
                    VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD,
                    dropdown_item_local_center_y(0, RESOLUTIONS.len()),
                    VIDEO_RESOLUTION_DROPDOWN_ARROW_Z,
                )
                .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
                Visibility::Hidden,
            ));
        });
        commands
            .entity(dropdown_entity)
            .insert(VideoResolutionDropdownValueArrowAttached);
    }
}

fn update_resolution_dropdown_value_arrows(
    settings: Res<VideoSettingsState>,
    mut arrow_query: Query<
        (
            &ChildOf,
            &VideoResolutionDropdownValueArrowSide,
            &mut Visibility,
            &mut Transform,
            &MeshMaterial2d<ColorMaterial>,
        ),
        With<VideoResolutionDropdownValueArrow>,
    >,
    dropdown_visibility_query: Query<
        &Visibility,
        (
            With<VideoResolutionDropdown>,
            Without<VideoResolutionDropdownValueArrow>,
        ),
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !settings.initialized {
        return;
    }

    let selected_index = settings.pending.resolution_index.min(RESOLUTIONS.len() - 1);
    let selected_y = dropdown_item_local_center_y(selected_index, RESOLUTIONS.len());

    for (parent, side, mut visibility, mut transform, material_handle) in arrow_query.iter_mut() {
        let Ok(dropdown_visibility) = dropdown_visibility_query.get(parent.parent()) else {
            continue;
        };

        transform.translation.x = match side {
            VideoResolutionDropdownValueArrowSide::Left => -VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD,
            VideoResolutionDropdownValueArrowSide::Right => VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD,
        };
        transform.translation.y = selected_y;
        transform.translation.z = VIDEO_RESOLUTION_DROPDOWN_ARROW_Z;

        *visibility = if *dropdown_visibility == Visibility::Visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if let Some(material) = materials.get_mut(material_handle.0.id()) {
            material.color = SYSTEM_MENU_COLOR;
        }
    }
}

fn recenter_resolution_dropdown_item_text(
    mut item_query: Query<
        (
            &VideoResolutionDropdownItemBaseY,
            &Anchor,
            &TextBounds,
            &TextLayoutInfo,
            &mut Transform,
        ),
        With<VideoResolutionDropdownItem>,
    >,
) {
    for (base_y, anchor, bounds, text_layout, mut transform) in item_query.iter_mut() {
        let height = bounds.height.unwrap_or(text_layout.size.y).max(1.0);

        let is_centered_vertical_anchor = matches!(
            anchor,
            &Anchor::CENTER_LEFT | &Anchor::CENTER | &Anchor::CENTER_RIGHT
        );

        let center_correction = if is_centered_vertical_anchor {
            let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
            for run in &text_layout.run_geometry {
                min_y = min_y.min(run.bounds.min.y);
                max_y = max_y.max(run.bounds.max.y);
            }
            if min_y.is_finite() && max_y.is_finite() {
                (min_y + max_y) * 0.5 - height * 0.5
            } else {
                0.0
            }
        } else {
            0.0
        };

        let target_y = base_y.0 + center_correction;
        if (transform.translation.y - target_y).abs() > 0.001 {
            transform.translation.y = target_y;
        }
    }
}

fn close_video_modals(commands: &mut Commands, modal_query: &Query<Entity, With<VideoModalRoot>>) {
    for modal_entity in modal_query.iter() {
        commands.entity(modal_entity).despawn_related::<Children>();
        commands.entity(modal_entity).despawn();
    }
}

fn apply_menu_intents(
    mut menu_intents: MessageReader<MenuIntent>,
    mut option_query: Query<(
        &Selectable,
        &MenuOptionCommand,
        &mut Clickable<SystemMenuActions>,
    ), Without<VideoModalButton>>,
    mut modal_button_query: Query<
        (&VideoModalButton, &mut Clickable<SystemMenuActions>),
        Without<MenuOptionCommand>,
    >,
) {
    let mut command_intents: Vec<(Entity, MenuCommand)> = Vec::new();
    let mut modal_button_intents: Vec<VideoModalButton> = Vec::new();

    for intent in menu_intents.read() {
        match intent {
            MenuIntent::TriggerCommand {
                menu_entity,
                command,
            } => command_intents.push((*menu_entity, command.clone())),
            MenuIntent::TriggerModalButton(button) => modal_button_intents.push(*button),
        }
    }

    for (menu_entity, command) in command_intents {
        for (selectable, option_command, mut clickable) in option_query.iter_mut() {
            if selectable.menu_entity == menu_entity && option_command.0 == command {
                clickable.triggered = true;
                break;
            }
        }
    }

    for target_button in modal_button_intents {
        for (button, mut clickable) in modal_button_query.iter_mut() {
            if *button == target_button {
                clickable.triggered = true;
                break;
            }
        }
    }
}

fn handle_video_modal_shortcuts(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<(), With<InteractionCapture>>,
    modal_query: Query<
        (
            Entity,
            Option<&VideoApplyConfirmModal>,
            Option<&VideoExitUnsavedModal>,
            Option<&InteractionGate>,
        ),
        With<VideoModalRoot>,
    >,
    mut menu_intents: MessageWriter<MenuIntent>,
) {
    let yes_pressed = keyboard_input.just_pressed(KeyCode::KeyY);
    let no_pressed = keyboard_input.just_pressed(KeyCode::KeyN);
    let cancel_pressed = keyboard_input.just_pressed(KeyCode::Backspace)
        || keyboard_input.just_pressed(KeyCode::Escape);

    if !(yes_pressed || no_pressed || cancel_pressed) {
        return;
    }

    let interaction_captured = interaction_context_active(pause_state.as_ref(), &capture_query);
    for (_, is_apply_modal, is_exit_modal, gate) in modal_query.iter() {
        if !interaction_gate_allows(gate, interaction_captured) {
            continue;
        }

        let target_button = if is_apply_modal.is_some() {
            if yes_pressed {
                Some(VideoModalButton::ApplyKeep)
            } else if no_pressed || cancel_pressed {
                Some(VideoModalButton::ApplyRevert)
            } else {
                None
            }
        } else if is_exit_modal.is_some() {
            if yes_pressed {
                Some(VideoModalButton::ExitWithoutSaving)
            } else if no_pressed || cancel_pressed {
                Some(VideoModalButton::ExitCancel)
            } else {
                None
            }
        } else {
            None
        };

        let Some(target_button) = target_button else {
            continue;
        };

        menu_intents.write(MenuIntent::TriggerModalButton(target_button));
        break;
    }
}

fn handle_video_modal_button_commands(
    mut commands: Commands,
    mut settings: ResMut<VideoSettingsState>,
    mut navigation_state: ResMut<MenuNavigationState>,
    mut button_query: Query<(
        Entity,
        &VideoModalButton,
        &mut Clickable<SystemMenuActions>,
        Option<&TransientAudioPallet<SystemMenuSounds>>,
    )>,
    modal_query: Query<Entity, With<VideoModalRoot>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    let mut clicked_button: Option<VideoModalButton> = None;
    for (entity, button, mut clickable, click_pallet) in button_query.iter_mut() {
        if clickable.triggered {
            clickable.triggered = false;
            if let Some(click_pallet) = click_pallet {
                TransientAudioPallet::play_transient_audio(
                    entity,
                    &mut commands,
                    click_pallet,
                    SystemMenuSounds::Click,
                    dilation.0,
                    &mut audio_query,
                );
            }
            clicked_button = Some(*button);
            break;
        }
    }

    let Some(button) = clicked_button else {
        return;
    };

    match button {
        VideoModalButton::ApplyKeep => {
            settings.saved = settings.pending;
            settings.revert_snapshot = None;
            settings.apply_timer = None;
        }
        VideoModalButton::ApplyRevert => {
            if let Some(snapshot) = settings.revert_snapshot.take() {
                settings.pending = snapshot;
                if let Ok(mut window) = primary_window.single_mut() {
                    apply_snapshot_to_window(&mut window, snapshot);
                }
            }
            settings.apply_timer = None;
        }
        VideoModalButton::ExitWithoutSaving => {
            settings.pending = settings.saved;
            navigation_state.pending_exit_menu = navigation_state.exit_prompt_target_menu.take();
            navigation_state.pending_exit_closes_menu_system =
                navigation_state.exit_prompt_closes_menu_system;
            navigation_state.exit_prompt_closes_menu_system = false;
        }
        VideoModalButton::ExitCancel => {
            navigation_state.exit_prompt_target_menu = None;
            navigation_state.pending_exit_menu = None;
            navigation_state.exit_prompt_closes_menu_system = false;
            navigation_state.pending_exit_closes_menu_system = false;
        }
    }

    close_video_modals(&mut commands, &modal_query);
}

fn handle_resolution_dropdown_item_commands(
    mut commands: Commands,
    mut settings: ResMut<VideoSettingsState>,
    mut dropdown_state: ResMut<MenuDropdownState>,
    mut item_query: Query<(
        Entity,
        &ChildOf,
        &VideoResolutionDropdownItem,
        &mut Clickable<SystemMenuActions>,
        Option<&TransientAudioPallet<SystemMenuSounds>>,
    )>,
    mut dropdown_query: Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    if !settings.initialized {
        return;
    }

    let mut visible_dropdowns = HashSet::new();
    for (dropdown_entity, _, visibility) in dropdown_query.iter_mut() {
        if *visibility == Visibility::Visible {
            visible_dropdowns.insert(dropdown_entity);
        }
    }

    let mut chosen_resolution: Option<usize> = None;
    for (entity, parent, item, mut clickable, click_pallet) in item_query.iter_mut() {
        if clickable.triggered {
            clickable.triggered = false;
            if !visible_dropdowns.contains(&parent.parent()) {
                continue;
            }
            if let Some(click_pallet) = click_pallet {
                TransientAudioPallet::play_transient_audio(
                    entity,
                    &mut commands,
                    click_pallet,
                    SystemMenuSounds::Click,
                    dilation.0,
                    &mut audio_query,
                );
            }
            chosen_resolution = Some(item.index);
            break;
        }
    }

    let Some(index) = chosen_resolution else {
        return;
    };

    settings.pending.resolution_index = index.min(RESOLUTIONS.len() - 1);
    dropdown_state.suppress_toggle_once = true;
    close_all_dropdowns(&mut dropdown_state, &mut dropdown_query);
}

fn close_resolution_dropdown_on_outside_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    settings: Res<VideoSettingsState>,
    mut dropdown_state: ResMut<MenuDropdownState>,
    mut dropdown_query: Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
    item_query: Query<&Clickable<SystemMenuActions>, With<VideoResolutionDropdownItem>>,
) {
    if !settings.initialized
        || dropdown_state.open_menu.is_none()
        || !mouse_input.just_pressed(MouseButton::Left)
    {
        return;
    }

    // If a dropdown row handled this click, keep normal item-selection flow.
    let clicked_item = item_query.iter().any(|clickable| clickable.triggered);
    if clicked_item {
        return;
    }

    let any_visible = dropdown_query
        .iter_mut()
        .any(|(_, _, visibility)| *visibility == Visibility::Visible);
    if !any_visible {
        return;
    }

    dropdown_state.suppress_toggle_once = true;
    close_all_dropdowns(&mut dropdown_state, &mut dropdown_query);
}

fn handle_resolution_dropdown_keyboard_navigation(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<(), With<InteractionCapture>>,
    mut settings: ResMut<VideoSettingsState>,
    mut dropdown_state: ResMut<MenuDropdownState>,
    mut selectable_menu_queries: ParamSet<(
        Query<(Entity, &MenuStack, &MenuRoot, &mut SelectableMenu)>,
        Query<&mut SelectableMenu, (With<VideoResolutionDropdown>, Without<MenuRoot>)>,
    )>,
    mut option_query: Query<
        (
            &Selectable,
            &mut InteractionVisualState,
            &mut Clickable<SystemMenuActions>,
        ),
        With<MenuOptionCommand>,
    >,
    mut dropdown_query: Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
    modal_query: Query<(), With<VideoModalRoot>>,
) {
    if !settings.initialized || any_video_modal_open(&modal_query) {
        return;
    }

    let left_pressed = keyboard_input.just_pressed(KeyCode::ArrowLeft);
    let right_pressed = keyboard_input.just_pressed(KeyCode::ArrowRight);
    let backspace_pressed = keyboard_input.just_pressed(KeyCode::Backspace);
    let escape_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    let resolution_shortcut = dropdown_resolution_shortcut_index(&keyboard_input);
    let dropdown_open = dropdown_state.open_menu.is_some();

    if !(left_pressed
        || right_pressed
        || backspace_pressed
        || escape_pressed
        || resolution_shortcut.is_some()
        || dropdown_open)
    {
        return;
    }

    let interaction_captured = interaction_context_active(pause_state.as_ref(), &capture_query);
    let resolution_option_index = video_resolution_option_index();
    let mut selected_resolution_menu: Option<Entity> = None;
    {
        let mut menu_query = selectable_menu_queries.p0();
        for (menu_entity, menu_stack, menu_root, mut selectable_menu) in menu_query.iter_mut() {
            if !interaction_gate_allows(Some(&menu_root.gate), interaction_captured) {
                continue;
            }
            if menu_stack.current_page() != Some(MenuPage::Video) {
                continue;
            }

            if dropdown_state.open_menu == Some(menu_entity) {
                selectable_menu.selected_index = resolution_option_index;
                for (selectable, mut visual_state, mut clickable) in option_query.iter_mut() {
                    if selectable.menu_entity != menu_entity {
                        continue;
                    }
                    clickable.triggered = false;
                    if selectable.index == resolution_option_index {
                        visual_state.selected = true;
                    } else {
                        visual_state.selected = false;
                        visual_state.hovered = false;
                        visual_state.pressed = false;
                    }
                }

                if let Some(resolution_index) = resolution_shortcut {
                    settings.pending.resolution_index = resolution_index.min(RESOLUTIONS.len() - 1);
                    dropdown_state.suppress_toggle_once = true;
                    close_dropdowns_for_menu(menu_entity, &mut dropdown_state, &mut dropdown_query);
                } else if (left_pressed && !right_pressed) || backspace_pressed || escape_pressed {
                    close_dropdowns_for_menu(menu_entity, &mut dropdown_state, &mut dropdown_query);
                }
                return;
            }

            if selectable_menu.selected_index == resolution_option_index
                && selected_resolution_menu.is_none()
            {
                selected_resolution_menu = Some(menu_entity);
            }
        }
    }

    if dropdown_state.open_menu.is_some() {
        close_all_dropdowns(&mut dropdown_state, &mut dropdown_query);
        return;
    }

    if right_pressed && !left_pressed && !backspace_pressed && !escape_pressed {
        if let Some(menu_entity) = selected_resolution_menu {
            let mut dropdown_menu_query = selectable_menu_queries.p1();
            open_dropdown_for_menu(
                menu_entity,
                settings.pending.resolution_index.min(RESOLUTIONS.len() - 1),
                &mut dropdown_state,
                &mut dropdown_query,
                &mut dropdown_menu_query,
            );
        }
    }
}

fn update_apply_confirmation_countdown(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut settings: ResMut<VideoSettingsState>,
    mut countdown_text_query: Query<&mut Text2d, With<VideoApplyCountdownText>>,
    modal_query: Query<Entity, With<VideoModalRoot>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(timer) = settings.apply_timer.as_mut() else {
        return;
    };

    timer.tick(time.delta());
    let remaining = (30.0 - timer.elapsed_secs()).ceil().max(0.0) as i32;
    for mut text in countdown_text_query.iter_mut() {
        text.0 = format!("Reverting in {remaining}");
    }

    if !timer.is_finished() {
        return;
    }

    if let Some(snapshot) = settings.revert_snapshot.take() {
        settings.pending = snapshot;
        if let Ok(mut window) = primary_window.single_mut() {
            apply_snapshot_to_window(&mut window, snapshot);
        }
    }
    settings.apply_timer = None;
    close_video_modals(&mut commands, &modal_query);
}

pub fn spawn_menu_root<B>(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    host: MenuHost,
    name: &str,
    translation: Vec3,
    initial_page: MenuPage,
    gate: InteractionGate,
    extra_components: B,
) -> Entity
where
    B: Bundle,
{
    let menu_entity = system_menu::spawn_root(
        commands,
        asset_server,
        name,
        translation,
        SystemMenuSounds::Switch,
        (
            MenuRoot { host, gate },
            MenuStack::new(initial_page),
            gate,
            extra_components,
        ),
    );

    spawn_page_content(commands, asset_server, menu_entity, initial_page, gate);

    if host == MenuHost::Pause {
        commands.entity(menu_entity).with_children(|parent| {
            parent.spawn((
                Name::new("pause_menu_music"),
                PauseMenuAudio,
                AudioPlayer::<AudioSource>(asset_server.load(PAUSE_MENU_MUSIC_PATH)),
                PlaybackSettings {
                    volume: Volume::Linear(0.25),
                    ..continuous_audio()
                },
            ));
        });
    }

    menu_entity
}

fn play_menu_navigation_sound(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<(), With<InteractionCapture>>,
    modal_query: Query<(), With<VideoModalRoot>>,
    modal_menu_query: Query<
        (
            Entity,
            &SelectableMenu,
            &TransientAudioPallet<SystemMenuSounds>,
            Option<&InteractionGate>,
        ),
        With<VideoModalRoot>,
    >,
    dropdown_visibility_query: Query<&Visibility, With<VideoResolutionDropdown>>,
    dropdown_menu_query: Query<
        (
            Entity,
            &SelectableMenu,
            &TransientAudioPallet<SystemMenuSounds>,
            &Visibility,
        ),
        With<VideoResolutionDropdown>,
    >,
    menu_query: Query<
        (
            Entity,
            &SelectableMenu,
            &TransientAudioPallet<SystemMenuSounds>,
        ),
        With<MenuRoot>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    let interaction_captured = interaction_context_active(pause_state.as_ref(), &capture_query);

    if any_video_modal_open(&modal_query) {
        for (modal_entity, menu, pallet, gate) in modal_menu_query.iter() {
            if !interaction_gate_allows(gate, interaction_captured) {
                continue;
            }
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
                    modal_entity,
                    &mut commands,
                    pallet,
                    SystemMenuSounds::Switch,
                    dilation.0,
                    &mut audio_query,
                );
            }
        }
        return;
    }

    if any_resolution_dropdown_open(&dropdown_visibility_query) {
        for (dropdown_entity, menu, pallet, visibility) in dropdown_menu_query.iter() {
            if *visibility != Visibility::Visible {
                continue;
            }
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
                    dropdown_entity,
                    &mut commands,
                    pallet,
                    SystemMenuSounds::Switch,
                    dilation.0,
                    &mut audio_query,
                );
            }
        }
        return;
    }

    system_menu::play_navigation_sound(
        &mut commands,
        keyboard_input.as_ref(),
        &menu_query,
        &mut audio_query,
        SystemMenuSounds::Switch,
        dilation.0,
    );
}

fn handle_menu_shortcuts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<(), With<InteractionCapture>>,
    settings: Res<VideoSettingsState>,
    dropdown_state: Res<MenuDropdownState>,
    mut navigation_state: ResMut<MenuNavigationState>,
    modal_query: Query<(), With<VideoModalRoot>>,
    menu_query: Query<(Entity, &MenuStack, &MenuRoot)>,
    option_shortcut_query: Query<(&Selectable, &MenuOptionShortcut, &MenuOptionCommand)>,
    mut menu_intents: MessageWriter<MenuIntent>,
) {
    if any_video_modal_open(&modal_query) || dropdown_state.open_menu.is_some() {
        return;
    }

    let escape_pressed = keyboard_input.just_pressed(KeyCode::Escape);

    let interaction_captured = interaction_context_active(pause_state.as_ref(), &capture_query);
    let mut handled_escape = false;

    for (menu_entity, menu_stack, menu_root) in menu_query.iter() {
        if !interaction_gate_allows(Some(&menu_root.gate), interaction_captured) {
            continue;
        }

        let Some(page) = menu_stack.current_page() else {
            continue;
        };

        if escape_pressed && !handled_escape {
            handled_escape = true;
            let leaving_video_options = matches!(page, MenuPage::Video | MenuPage::Options);
            if settings.initialized && video_settings_dirty(&settings) && leaving_video_options {
                navigation_state.exit_prompt_target_menu = Some(menu_entity);
                navigation_state.exit_prompt_closes_menu_system = true;
                spawn_exit_unsaved_modal(&mut commands, menu_entity, &asset_server, menu_root.gate);
            } else {
                navigation_state.pending_exit_menu = Some(menu_entity);
                navigation_state.pending_exit_closes_menu_system = true;
            }
            continue;
        }
    }

    let pending_shortcuts = selector::collect_shortcut_commands(
        &keyboard_input,
        interaction_captured,
        &menu_query,
        &option_shortcut_query,
    );
    for (menu_entity, command) in pending_shortcuts {
        menu_intents.write(MenuIntent::TriggerCommand {
            menu_entity,
            command,
        });
    }
}

fn handle_menu_option_commands(
    mut commands: Commands,
    mut option_query: Query<(
        Entity,
        &Selectable,
        &mut Clickable<SystemMenuActions>,
        &MenuOptionCommand,
        &TransientAudioPallet<SystemMenuSounds>,
    )>,
    mut menu_queries: ParamSet<(
        Query<(&MenuRoot, &mut MenuStack, &mut SelectableMenu)>,
        Query<(&MenuRoot, &MenuStack)>,
        Query<&mut SelectableMenu, (With<VideoResolutionDropdown>, Without<MenuRoot>)>,
    )>,
    page_content_query: Query<(Entity, &ChildOf), With<MenuPageContent>>,
    asset_server: Res<AssetServer>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
    mut window_exit: WindowExitContext,
    mut next_main_state: ResMut<NextState<MainState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
    mut settings: ResMut<VideoSettingsState>,
    mut dropdown_state: ResMut<MenuDropdownState>,
    mut navigation_state: ResMut<MenuNavigationState>,
    mut dropdown_query: Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
    modal_query: Query<(), With<VideoModalRoot>>,
) {
    let mut dirty_menus = HashSet::new();
    let mut closed_menus = HashSet::new();
    let mut pending_dropdown_open: Vec<(Entity, usize)> = Vec::new();
    let mut menu_query = menu_queries.p0();
    let modal_open = any_video_modal_open(&modal_query);
    let mut suppress_resolution_toggle = dropdown_state.suppress_toggle_once;
    dropdown_state.suppress_toggle_once = false;

    if let Some(menu_entity) = navigation_state.pending_exit_menu.take() {
        let close_menu_system = navigation_state.pending_exit_closes_menu_system;
        navigation_state.pending_exit_closes_menu_system = false;

        if let Ok((menu_root, mut menu_stack, mut selectable_menu)) = menu_query.get_mut(menu_entity) {
            if close_menu_system {
                match menu_root.host {
                    MenuHost::Pause => next_pause_state.set(PauseState::Unpaused),
                    MenuHost::Debug | MenuHost::Main => {
                        closed_menus.insert(menu_entity);
                    }
                }
            } else {
                if let Some(selected_index) = menu_stack.pop() {
                    selectable_menu.selected_index = selected_index;
                    dirty_menus.insert(menu_entity);
                } else {
                    closed_menus.insert(menu_entity);
                }
            }
        }
    }

    for (option_entity, selectable, mut clickable, option_command, click_pallet) in
        option_query.iter_mut()
    {
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;

        if modal_open {
            continue;
        }

        if dropdown_state.open_menu == Some(selectable.menu_entity)
            && !matches!(&option_command.0, MenuCommand::ToggleResolutionDropdown)
        {
            continue;
        }

        TransientAudioPallet::play_transient_audio(
            option_entity,
            &mut commands,
            click_pallet,
            SystemMenuSounds::Click,
            dilation.0,
            &mut audio_query,
        );

        let Ok((menu_root, mut menu_stack, mut selectable_menu)) =
            menu_query.get_mut(selectable.menu_entity)
        else {
            continue;
        };

        menu_stack.remember_selected_index(selectable_menu.selected_index);
        let current_page = menu_stack.current_page();

        match option_command.0.clone() {
            MenuCommand::None => {}
            MenuCommand::Push(page) => {
                close_dropdowns_for_menu(
                    selectable.menu_entity,
                    &mut dropdown_state,
                    &mut dropdown_query,
                );
                menu_stack.push(page);
                selectable_menu.selected_index = menu_stack.current_selected_index();
                dirty_menus.insert(selectable.menu_entity);
            }
            MenuCommand::Pop => {
                close_dropdowns_for_menu(
                    selectable.menu_entity,
                    &mut dropdown_state,
                    &mut dropdown_query,
                );
                let leaving_video_options = current_page
                    .is_some_and(|page| matches!(page, MenuPage::Video | MenuPage::Options));
                if settings.initialized && video_settings_dirty(&settings) && leaving_video_options
                {
                    navigation_state.exit_prompt_target_menu = Some(selectable.menu_entity);
                    navigation_state.exit_prompt_closes_menu_system = false;
                    spawn_exit_unsaved_modal(
                        &mut commands,
                        selectable.menu_entity,
                        &asset_server,
                        menu_root.gate,
                    );
                } else {
                    if let Some(selected_index) = menu_stack.pop() {
                        selectable_menu.selected_index = selected_index;
                        dirty_menus.insert(selectable.menu_entity);
                    } else {
                        closed_menus.insert(selectable.menu_entity);
                    }
                }
            }
            MenuCommand::ToggleResolutionDropdown => {
                if !settings.initialized {
                    continue;
                }
                if suppress_resolution_toggle {
                    suppress_resolution_toggle = false;
                    continue;
                }
                if dropdown_state.open_menu == Some(selectable.menu_entity) {
                    close_dropdowns_for_menu(
                        selectable.menu_entity,
                        &mut dropdown_state,
                        &mut dropdown_query,
                    );
                } else {
                    pending_dropdown_open.push((
                        selectable.menu_entity,
                        settings.pending.resolution_index.min(RESOLUTIONS.len() - 1),
                    ));
                }
            }
            MenuCommand::SetPause(state) => {
                next_pause_state.set(state);
            }
            MenuCommand::ToggleDisplayMode => {
                if settings.initialized {
                    close_dropdowns_for_menu(
                        selectable.menu_entity,
                        &mut dropdown_state,
                        &mut dropdown_query,
                    );
                    settings.pending.display_mode = match settings.pending.display_mode {
                        VideoDisplayMode::Windowed => VideoDisplayMode::Borderless,
                        VideoDisplayMode::Borderless => VideoDisplayMode::Windowed,
                    };
                }
            }
            MenuCommand::ToggleVsync => {
                if settings.initialized {
                    close_dropdowns_for_menu(
                        selectable.menu_entity,
                        &mut dropdown_state,
                        &mut dropdown_query,
                    );
                    settings.pending.vsync_enabled = !settings.pending.vsync_enabled;
                }
            }
            MenuCommand::ApplyVideoSettings => {
                if settings.initialized && video_settings_dirty(&settings) {
                    close_dropdowns_for_menu(
                        selectable.menu_entity,
                        &mut dropdown_state,
                        &mut dropdown_query,
                    );
                    settings.revert_snapshot = Some(settings.saved);
                    if let Ok(mut window) = window_exit.primary_window_queries.p1().single_mut() {
                        apply_snapshot_to_window(&mut window, settings.pending);
                    }
                    settings.apply_timer = Some(Timer::from_seconds(30.0, TimerMode::Once));
                    spawn_apply_confirm_modal(
                        &mut commands,
                        selectable.menu_entity,
                        &asset_server,
                        menu_root.gate,
                    );
                }
            }
            MenuCommand::ResetVideoDefaults => {
                if settings.initialized {
                    settings.pending = default_video_settings();
                    close_dropdowns_for_menu(
                        selectable.menu_entity,
                        &mut dropdown_state,
                        &mut dropdown_query,
                    );
                }
            }
            MenuCommand::SetMain(state) => {
                next_main_state.set(state);
            }
            MenuCommand::SetPauseAndMain(pause_state, main_state) => {
                next_pause_state.set(pause_state);
                next_main_state.set(main_state);
            }
            MenuCommand::ExitApplication => {
                let primary_window = window_exit.primary_window_queries.p0();
                request_application_exit(
                    &primary_window,
                    &mut window_exit.close_requests,
                    &mut window_exit.app_exit,
                );
            }
            MenuCommand::CloseMenu => {
                closed_menus.insert(selectable.menu_entity);
            }
        }
    }

    for (menu_entity, selected_index) in pending_dropdown_open {
        if closed_menus.contains(&menu_entity) {
            continue;
        }
        let mut dropdown_menu_query = menu_queries.p2();
        open_dropdown_for_menu(
            menu_entity,
            selected_index,
            &mut dropdown_state,
            &mut dropdown_query,
            &mut dropdown_menu_query,
        );
    }

    if navigation_state
        .exit_prompt_target_menu
        .is_some_and(|menu_entity| closed_menus.contains(&menu_entity))
    {
        navigation_state.exit_prompt_target_menu = None;
    }

    for menu_entity in closed_menus {
        if dropdown_state.open_menu == Some(menu_entity) {
            dropdown_state.open_menu = None;
        }
        dirty_menus.remove(&menu_entity);
        commands.entity(menu_entity).despawn_related::<Children>();
        commands.entity(menu_entity).despawn();
    }

    let menu_query = menu_queries.p1();
    for menu_entity in dirty_menus {
        let Ok((menu_root, menu_stack)) = menu_query.get(menu_entity) else {
            continue;
        };
        let Some(current_page) = menu_stack.current_page() else {
            continue;
        };

        rebuild_menu_page(
            &mut commands,
            &asset_server,
            menu_entity,
            current_page,
            menu_root.gate,
            &page_content_query,
        );
    }
}

fn handle_option_cycler_commands(
    mut commands: Commands,
    mut option_query: Query<(
        Entity,
        &mut OptionCycler,
        &MenuOptionCommand,
        &TransientAudioPallet<SystemMenuSounds>,
    )>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
    mut settings: ResMut<VideoSettingsState>,
    dropdown_state: Res<MenuDropdownState>,
    modal_query: Query<(), With<VideoModalRoot>>,
) {
    let modal_open = any_video_modal_open(&modal_query);

    for (entity, mut cycler, option_command, click_pallet) in option_query.iter_mut() {
        let direction = if cycler.left_triggered {
            Some(false)
        } else if cycler.right_triggered {
            Some(true)
        } else {
            None
        };

        cycler.left_triggered = false;
        cycler.right_triggered = false;

        let Some(_forward) = direction else {
            continue;
        };

        if modal_open || dropdown_state.open_menu.is_some() || !settings.initialized {
            continue;
        }

        TransientAudioPallet::play_transient_audio(
            entity,
            &mut commands,
            click_pallet,
            SystemMenuSounds::Click,
            dilation.0,
            &mut audio_query,
        );

        match &option_command.0 {
            MenuCommand::ToggleDisplayMode => {
                settings.pending.display_mode = match settings.pending.display_mode {
                    VideoDisplayMode::Windowed => VideoDisplayMode::Borderless,
                    VideoDisplayMode::Borderless => VideoDisplayMode::Windowed,
                };
            }
            MenuCommand::ToggleVsync => {
                settings.pending.vsync_enabled = !settings.pending.vsync_enabled;
            }
            _ => {}
        }
    }
}

fn sanitize_menu_selection_indices(
    mut menu_query: Query<
        (Entity, &mut SelectableMenu),
        Or<(
            With<MenuRoot>,
            With<VideoResolutionDropdown>,
            With<VideoModalRoot>,
        )>,
    >,
    selectable_query: Query<&Selectable>,
) {
    let mut indices_by_menu: HashMap<Entity, Vec<usize>> = HashMap::new();

    for selectable in selectable_query.iter() {
        indices_by_menu
            .entry(selectable.menu_entity)
            .or_default()
            .push(selectable.index);
    }

    for (menu_entity, mut menu) in menu_query.iter_mut() {
        let Some(indices) = indices_by_menu.get_mut(&menu_entity) else {
            if menu.selected_index != 0 {
                menu.selected_index = 0;
            }
            continue;
        };

        indices.sort_unstable();
        indices.dedup();
        if indices.is_empty() {
            menu.selected_index = 0;
            continue;
        }

        if !indices.contains(&menu.selected_index) {
            menu.selected_index = indices[0];
        }
    }
}

fn enforce_menu_layer_invariants(
    mut navigation_state: ResMut<MenuNavigationState>,
    mut dropdown_state: ResMut<MenuDropdownState>,
    menu_root_query: Query<Entity, With<MenuRoot>>,
    modal_query: Query<(), With<VideoModalRoot>>,
    mut dropdown_query: Query<(Entity, &ChildOf, &mut Visibility), With<VideoResolutionDropdown>>,
) {
    stack::clear_stale_menu_targets(&mut navigation_state, &menu_root_query);
    dropdown::enforce_single_visible_layer::<VideoResolutionDropdown>(
        &mut dropdown_state,
        &menu_root_query,
        any_video_modal_open(&modal_query),
        &mut dropdown_query,
    );
}

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VideoSettingsState>()
            .init_resource::<MenuDropdownState>()
            .init_resource::<MenuNavigationState>()
            .add_message::<MenuIntent>();
        app.configure_sets(
            Update,
            (
                MenuSystems::Core,
                MenuSystems::Commands.after(MenuSystems::Core),
                MenuSystems::PostCommands.after(MenuSystems::Commands),
                MenuSystems::Visual.after(MenuSystems::PostCommands),
            ),
        );
        app.add_systems(
            Update,
            (
                initialize_video_settings_from_window,
                selector::sync_option_cycler_bounds,
                option_cycler_input_system,
                play_menu_navigation_sound,
                handle_menu_shortcuts,
                handle_resolution_dropdown_keyboard_navigation,
                apply_menu_intents,
            )
                .chain()
                .in_set(MenuSystems::Core)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                handle_video_modal_button_commands,
                handle_resolution_dropdown_item_commands,
                close_resolution_dropdown_on_outside_click,
                update_apply_confirmation_countdown,
                handle_menu_option_commands,
            )
                .chain()
                .in_set(MenuSystems::Commands)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                handle_option_cycler_commands,
                sanitize_menu_selection_indices,
                enforce_menu_layer_invariants,
            )
                .chain()
                .in_set(MenuSystems::PostCommands)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            handle_video_modal_shortcuts
                .after(option_cycler_input_system)
                .before(apply_menu_intents)
                .in_set(MenuSystems::Core)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                ensure_resolution_dropdown_value_arrows,
                update_resolution_dropdown_value_arrows,
                recenter_resolution_dropdown_item_text,
            )
                .chain()
                .in_set(MenuSystems::Visual)
                .after(sync_resolution_dropdown_items)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                sync_video_table_values,
                sync_resolution_dropdown_items,
                system_menu::ensure_selection_indicators,
                system_menu::update_selection_indicators,
                system_menu::ensure_selection_bars,
                system_menu::update_selection_bars,
                system_menu::ensure_cycle_arrows,
                system_menu::update_cycle_arrows,
            )
                .chain()
                .in_set(MenuSystems::Visual)
                .after(InteractionSystem::Selectable),
        );
    }
}
