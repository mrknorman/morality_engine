use super::*;
use bevy::sprite::Anchor;
use bevy::text::TextBounds;
use crate::{
    entities::{sprites::compound::HollowRectangle, text::TextButton},
    systems::{
        colors::SYSTEM_MENU_COLOR,
        ui::{
            discrete_slider::DiscreteSlider,
            dropdown::DropdownSurface,
            hover_box,
            selector::SelectorSurface,
            scroll::{
                ScrollAxis, ScrollBar, ScrollableContent, ScrollableContentExtent, ScrollableItem,
                ScrollableRoot, ScrollableViewport,
            },
        },
    },
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
    let mut video_top_options_parent = None;
    let mut video_option_hover_box_root = None;

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
            let hover_box_clamp_size = Vec2::new(
                (page_def.layout.panel_size.x - VIDEO_HOVER_BOX_CLAMP_INSET.x * 2.0).max(120.0),
                (page_def.layout.panel_size.y - VIDEO_HOVER_BOX_CLAMP_INSET.y * 2.0).max(120.0),
            );
            let hover_box_style = hover_box::HoverBoxStyle::default()
                .with_size(VIDEO_HOVER_BOX_SIZE)
                .with_font_size(VIDEO_HOVER_BOX_TEXT_SIZE);
            let option_hover_root = hover_box::spawn_hover_box_root(
                parent,
                "system_video_option_hover_box",
                menu_entity,
                UiLayerKind::Base,
                hover_box_clamp_size,
                hover_box_style,
                VIDEO_HOVER_BOX_DELAY_SECONDS,
                (MenuPageContent, VideoOptionHoverBoxRoot),
            );
            let dropdown_hover_style = hover_box_style.with_z(VIDEO_DROPDOWN_HOVER_BOX_Z);
            let dropdown_hover_root = hover_box::spawn_hover_box_root(
                parent,
                "system_video_dropdown_hover_box",
                menu_entity,
                UiLayerKind::Dropdown,
                hover_box_clamp_size,
                dropdown_hover_style,
                VIDEO_HOVER_BOX_DELAY_SECONDS,
                (MenuPageContent, VideoDropdownHoverBoxRoot),
            );
            video_option_hover_box_root = Some(option_hover_root);

            let top_scroll_root = parent
                .spawn((
                    Name::new("system_video_top_options_scroll_root"),
                    MenuPageContent,
                    VideoTopOptionsScrollRoot,
                    gate,
                    ScrollableRoot::new(menu_entity, ScrollAxis::Vertical)
                        .with_edge_zones(12.0, VIDEO_TOP_SCROLL_VIEWPORT_VERTICAL_INSET)
                        .with_input_layer(UiLayerKind::Base),
                    ScrollableViewport::new(Vec2::new(
                        VIDEO_TABLE_TOTAL_WIDTH,
                        VIDEO_TOP_SCROLL_VIEWPORT_HEIGHT,
                    )),
                    ScrollableContentExtent(VIDEO_TOP_SCROLL_CONTENT_HEIGHT),
                    scroll_adapter::ScrollableTableAdapter::new(
                        menu_entity,
                        VIDEO_TOP_OPTION_COUNT,
                        VIDEO_TABLE_ROW_HEIGHT,
                        VIDEO_TOP_SCROLL_LEADING_PADDING,
                    ),
                    Transform::from_xyz(0.0, VIDEO_TOP_SCROLL_CENTER_Y, VIDEO_TOP_SCROLL_Z),
                ))
                .id();

            let top_scroll_content = parent
                .spawn((
                    Name::new("system_video_top_options_scroll_content"),
                    MenuPageContent,
                    VideoTopOptionsScrollContent,
                    ScrollableContent,
                    Transform::default(),
                ))
                .id();

            parent.commands().entity(top_scroll_root).add_child(top_scroll_content);
            parent.commands().entity(top_scroll_root).with_children(|scroll_root| {
                scroll_root.spawn((
                    Name::new("system_video_top_options_scrollbar"),
                    MenuPageContent,
                    ScrollBar::new(top_scroll_root),
                    Transform::from_xyz(0.0, 0.0, 0.12),
                ));
            });

            parent.commands().entity(top_scroll_content).with_children(|content| {
                content.spawn((
                    Name::new("system_video_top_options_table"),
                    MenuPageContent,
                    VideoTopOptionsTable,
                    video_top_options_table(),
                    Transform::from_xyz(
                        VIDEO_TABLE_X,
                        video_top_scroll_local_y(VIDEO_TABLE_Y),
                        0.95,
                    ),
                ));
            });

            video_top_options_parent = Some(top_scroll_content);

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
                        SelectorSurface::new(tabs_entity, tab_index),
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

            let resolution_row_index = video_resolution_option_index();
            let dropdown_rows = VIDEO_DROPDOWN_MAX_ROWS;
            let dropdown_height = dropdown_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
            let dropdown_entity = parent
                .spawn((
                    Name::new("system_video_resolution_dropdown"),
                    MenuPageContent,
                    VideoResolutionDropdown,
                    DropdownSurface::new(menu_entity)
                        .with_click_activation(SelectableClickActivation::HoveredOnly),
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

                    for index in 0..VIDEO_DROPDOWN_MAX_ROWS {
                        let base_y = dropdown_item_local_center_y(index, dropdown_rows);
                        let dropdown_item_entity = dropdown
                            .spawn((
                                Name::new(format!("system_video_resolution_dropdown_item_{index}")),
                                MenuPageContent,
                                VideoResolutionDropdownItem { index },
                                TextButton,
                                gate,
                                SelectorSurface::new(dropdown_entity, index),
                                Clickable::with_region(
                                    vec![SystemMenuActions::Activate],
                                    Vec2::new(
                                        VIDEO_RESOLUTION_DROPDOWN_WIDTH - 8.0,
                                        VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT,
                                    ),
                                ),
                                Text2d::new(VIDEO_MENU_VALUE_PLACEHOLDER.to_string()),
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
                            .id();
                        dropdown.commands().entity(dropdown_item_entity).insert((
                            VideoResolutionDropdownItemBaseY(base_y),
                            hover_box::HoverBoxTarget::new(
                                dropdown_hover_root,
                                Vec2::new(
                                    VIDEO_RESOLUTION_DROPDOWN_WIDTH - 8.0,
                                    VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT,
                                ),
                            ),
                            hover_box::HoverBoxContent::default(),
                        ));
                    }
                });
        });
    }

    let top_parent = video_top_options_parent.unwrap_or(menu_entity);
    for (index, option) in page_def.options.iter().enumerate() {
        let option_label = if is_video_page { "" } else { option.label };
        let mut option_position = if is_video_page {
            video_option_position(index)
        } else {
            Vec2::new(0.0, option.y)
        };
        let target_parent = if is_video_page && index < VIDEO_TOP_OPTION_COUNT {
            option_position.y = video_top_scroll_local_y(option_position.y);
            top_parent
        } else {
            menu_entity
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

            let mut entity_id = None;
            commands.entity(target_parent).with_children(|parent| {
                let mut entity = parent.spawn((
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
                ));
                if index < VIDEO_TOP_OPTION_COUNT {
                    entity.insert(ScrollableItem::new(
                        index as u64 + 1,
                        index,
                        VIDEO_TABLE_ROW_HEIGHT,
                    ));
                    if let Some(hover_root) = video_option_hover_box_root {
                        entity.insert((
                            hover_box::HoverBoxTarget::new(
                                hover_root,
                                Vec2::new(VIDEO_TABLE_LABEL_COLUMN_WIDTH, VIDEO_OPTION_REGION_HEIGHT),
                            )
                            .with_hover_region(
                                Vec2::new(VIDEO_TABLE_LABEL_COLUMN_WIDTH, VIDEO_OPTION_REGION_HEIGHT),
                                Vec2::new(VIDEO_NAME_COLUMN_CENTER_X, 0.0),
                            ),
                            hover_box::HoverBoxContent::default(),
                        ));
                    }
                }
                entity_id = Some(entity.id());
            });

            let Some(entity_id) = entity_id else {
                continue;
            };

            if let Some(shortcut) = option.shortcut {
                commands.entity(entity_id).insert(ShortcutKey(shortcut));
            }

            if option.cyclable {
                commands
                    .entity(entity_id)
                    .insert(SelectorSurface::new(menu_entity, index).with_cycler());
            }

            if index < VIDEO_TOP_OPTION_COUNT {
                spawn_video_option_slider_widget(commands.reborrow(), entity_id, menu_entity, index);
            }
        } else {
            let mut entity_id = None;
            commands.entity(target_parent).with_children(|parent| {
                entity_id = Some(
                    parent
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
                        .id(),
                );
            });

            if let Some(shortcut) = option.shortcut {
                if let Some(entity_id) = entity_id {
                    commands.entity(entity_id).insert(ShortcutKey(shortcut));
                }
            }
        }
    }
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
