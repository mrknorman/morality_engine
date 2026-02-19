use super::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use crate::{
    entities::{sprites::compound::HollowRectangle, text::TextButton},
    systems::{colors::SYSTEM_MENU_COLOR, ui::discrete_slider::DiscreteSlider},
};

fn spawn_video_option_slider_widget(
    mut commands: Commands,
    option_entity: Entity,
    menu_entity: Entity,
    row: usize,
) {
    commands.entity(option_entity).with_children(|option_parent| {
        let slider_entity = option_parent
            .spawn((
                Name::new(format!("system_video_option_slider_{row}")),
                MenuPageContent,
                VideoOptionDiscreteSlider { menu_entity, row },
                DiscreteSlider::new(VIDEO_DISCRETE_SLIDER_MAX_STEPS, 0)
                    .with_layout_steps(VIDEO_DISCRETE_SLIDER_MAX_STEPS)
                    .with_slot_size(VIDEO_DISCRETE_SLIDER_SLOT_SIZE)
                    .with_slot_gap(VIDEO_DISCRETE_SLIDER_GAP)
                    .with_fill_color(Color::NONE)
                    .with_empty_color(Color::NONE)
                    .with_border_color(Color::NONE)
                    .with_border_thickness(2.0)
                    .with_fill_inset(3.0),
                Transform::from_xyz(VIDEO_DISCRETE_SLIDER_ROOT_X, 0.0, VIDEO_DISCRETE_SLIDER_Z),
                Visibility::Hidden,
            ))
            .id();

        option_parent.commands().entity(slider_entity).with_children(|slider| {
            slider.spawn((
                Name::new(format!("system_video_option_slider_label_{row}")),
                MenuPageContent,
                VideoOptionDiscreteSliderLabel,
                Text2d::new(""),
                TextFont {
                    font_size: VIDEO_TABLE_TEXT_SIZE,
                    ..default()
                },
                TextColor(SYSTEM_MENU_COLOR),
                Anchor::CENTER_LEFT,
                Transform::from_xyz(VIDEO_DISCRETE_SLIDER_LABEL_OFFSET_X, 0.0, 0.02),
                Visibility::Hidden,
            ));
        });
    });
}

pub(super) fn spawn_page_content(
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
                Name::new("system_video_top_options_table"),
                MenuPageContent,
                VideoTopOptionsTable,
                video_top_options_table(),
                Transform::from_xyz(VIDEO_TABLE_X, VIDEO_TABLE_Y, 0.95),
            ));
            parent.spawn((
                Name::new("system_video_tabs_table"),
                MenuPageContent,
                VideoTabsTable,
                video_tabs_table(),
                Transform::from_xyz(VIDEO_TABLE_X, VIDEO_TABS_TABLE_Y, 0.95),
            ));
            parent.spawn((
                Name::new("system_video_footer_options_table"),
                MenuPageContent,
                VideoFooterOptionsTable,
                video_footer_options_table(),
                Transform::from_xyz(VIDEO_TABLE_X, VIDEO_FOOTER_TABLE_Y, 0.95),
            ));
            parent.spawn((
                Name::new("system_video_footer_separator"),
                MenuPageContent,
                Sprite::from_color(
                    SYSTEM_MENU_COLOR,
                    Vec2::new(VIDEO_TABLE_TOTAL_WIDTH, VIDEO_FOOTER_SEPARATOR_THICKNESS),
                ),
                Transform::from_xyz(0.0, VIDEO_FOOTER_SEPARATOR_Y, 0.96),
            ));

            let tabs_entity = parent
                .spawn((
                    Name::new("system_video_tabs_interaction_root"),
                    MenuPageContent,
                    VideoTabsInteractionRoot,
                    gate,
                    tabs::TabBar::new(menu_entity),
                    tabs::TabBarState::default(),
                    tabs::TabBarStateSync::Explicit,
                    tabs::TabActivationPolicy::default(),
                    tabbed_menu::TabbedMenuConfig::new(
                        VIDEO_TOP_OPTION_COUNT,
                        VIDEO_FOOTER_OPTION_START_INDEX,
                        VIDEO_FOOTER_OPTION_COUNT,
                    ),
                    SelectableMenu::new(0, vec![], vec![], vec![], true)
                        .with_click_activation(SelectableClickActivation::HoveredOnly),
                    system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
                    Transform::from_xyz(0.0, 0.0, 1.1),
                ))
                .id();

            let tab_hitbox_width = VIDEO_TABLE_TOTAL_WIDTH / VIDEO_TABS.len() as f32 - 8.0;
            let tab_hitbox_height = VIDEO_TABS_ROW_HEIGHT - 4.0;
            parent.commands().entity(tabs_entity).with_children(|tabs_parent| {
                for tab_index in 0..VIDEO_TABS.len() {
                    tabs_parent.spawn((
                        Name::new(format!("system_video_tab_option_{tab_index}")),
                        MenuPageContent,
                        gate,
                        VideoTabOption { index: tab_index },
                        tabs::TabItem { index: tab_index },
                        Selectable::new(tabs_entity, tab_index),
                        Clickable::with_region(
                            vec![SystemMenuActions::Activate],
                            Vec2::new(tab_hitbox_width, tab_hitbox_height),
                        ),
                        system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
                        Transform::from_xyz(
                            video_tab_column_center_x(tab_index),
                            video_tabs_row_center_y(),
                            0.0,
                        ),
                    ));
                }
            });

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
                    UiLayer::new(menu_entity, UiLayerKind::Dropdown),
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
                        Vec2::new(
                            VIDEO_RESOLUTION_DROPDOWN_WIDTH
                                + VIDEO_RESOLUTION_DROPDOWN_BACKGROUND_PAD_X * 2.0,
                            dropdown_height,
                        ),
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
                        VideoResolutionDropdownBorder,
                        HollowRectangle {
                            dimensions: Vec2::new(
                                VIDEO_RESOLUTION_DROPDOWN_WIDTH
                                    + VIDEO_RESOLUTION_DROPDOWN_BACKGROUND_PAD_X * 2.0
                                    - 6.0,
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
            let option_position = if is_video_page {
                video_option_position(index)
            } else {
                Vec2::new(0.0, option.y)
            };
            let clickable = if is_video_page {
                Clickable::with_region(vec![SystemMenuActions::Activate], video_option_region(index))
            } else {
                Clickable::new(vec![SystemMenuActions::Activate])
            };

            if is_video_page {
                let mut visual_style = if index < VIDEO_TOP_OPTION_COUNT {
                    system_menu::SystemMenuOptionVisualStyle::default()
                        .without_selection_indicators()
                        .with_selection_bar(system_menu::SystemMenuSelectionBarStyle {
                            offset: Vec3::new(
                                VIDEO_NAME_COLUMN_CENTER_X,
                                0.0,
                                VIDEO_NAME_HIGHLIGHT_Z,
                            ),
                            size: Vec2::new(VIDEO_NAME_HIGHLIGHT_WIDTH, VIDEO_NAME_HIGHLIGHT_HEIGHT),
                            color: SYSTEM_MENU_COLOR,
                        })
                } else {
                    system_menu::SystemMenuOptionVisualStyle::default()
                        .with_indicator_offset(VIDEO_FOOTER_INDICATOR_X)
                };
                if option.cyclable && index < VIDEO_TOP_OPTION_COUNT {
                    visual_style = visual_style.with_cycle_arrows(VIDEO_VALUE_COLUMN_CENTER_X);
                }

                let entity_id = parent
                    .spawn((
                        Name::new(option.name),
                        MenuPageContent,
                        gate,
                        tabbed_menu::TabbedMenuOption::new(menu_entity),
                        VideoOptionRow { index },
                        system_menu::SystemMenuOptionBundle::new_at(
                            option_label,
                            option_position.x,
                            option_position.y,
                            menu_entity,
                            index,
                        )
                        .with_visual_style(visual_style),
                        clickable,
                        MenuOptionCommand(option.command.clone()),
                        click_audio(),
                    ))
                    .id();

                if let Some(shortcut) = option.shortcut {
                    parent
                        .commands()
                        .entity(entity_id)
                        .insert(ShortcutKey(shortcut));
                }

                if option.cyclable {
                    parent.commands().entity(entity_id).insert(OptionCycler::default());
                }

                if index < VIDEO_TOP_OPTION_COUNT {
                    spawn_video_option_slider_widget(
                        parent.commands(),
                        entity_id,
                        menu_entity,
                        index,
                    );
                }
            } else {
                let entity_id = parent
                    .spawn((
                    Name::new(option.name),
                    MenuPageContent,
                    gate,
                    system_menu::SystemMenuOptionBundle::new_at(
                        option_label,
                        option_position.x,
                        option_position.y,
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
                        .insert(ShortcutKey(shortcut));
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

pub(super) fn rebuild_menu_page(
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
