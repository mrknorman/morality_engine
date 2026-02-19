use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::*;
use crate::{
    entities::{
        sprites::compound::RectangleSides,
        text::{Cell, Column, Table},
    },
    systems::{
        colors::SYSTEM_MENU_COLOR,
        interaction::Clickable,
        ui::discrete_slider::{DiscreteSlider, DiscreteSliderSlot},
    },
};

pub(super) fn sync_video_top_table_values(
    settings: Res<VideoSettingsState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    video_option_query: Query<(&Selectable, &VideoOptionRow, &InteractionVisualState)>,
    table_query: Query<(&ChildOf, &Children), With<VideoTopOptionsTable>>,
    column_children_query: Query<&Children, With<Column>>,
    cell_children_query: Query<&Children, With<Cell>>,
    mut text_query: Query<(&mut Text2d, &mut TextColor, &mut TextFont, &mut Transform)>,
) {
    if !settings.initialized {
        return;
    }

    let mut selected_by_menu_row: HashMap<(Entity, usize), bool> = HashMap::new();
    for (selectable, row, state) in video_option_query.iter() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        selected_by_menu_row.insert(
            (selectable.menu_entity, row.index),
            state.selected || state.pressed,
        );
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    for (table_parent, table_children) in table_query.iter() {
        let menu_entity = table_parent.parent();
        let active_tab = video_tab_kind(active_tabs.get(&menu_entity).copied().unwrap_or(0));
        let labels = video_top_option_labels(active_tab);
        let value_strings = video_top_value_strings(settings.pending, active_tab);
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
                        if video_top_option_uses_slider(active_tab, row_index) {
                            if !text.0.is_empty() {
                                text.0.clear();
                            }
                        } else {
                            let Some(value) = value_strings.get(row_index) else {
                                break;
                            };
                            if text.0 != *value {
                                text.0 = value.clone();
                            }
                        }
                    } else if column_index == 0 {
                        let Some(label) = labels.get(row_index) else {
                            break;
                        };
                        if text.0 != *label {
                            text.0 = (*label).to_string();
                        }
                    }

                    let selected = selected_by_menu_row
                        .get(&(menu_entity, row_index))
                        .copied()
                        .unwrap_or(false);
                    if selected {
                        font.font_size = VIDEO_TABLE_TEXT_SELECTED_SIZE;
                        font.weight = FontWeight::BOLD;
                    } else {
                        font.font_size = VIDEO_TABLE_TEXT_SIZE;
                        font.weight = FontWeight::NORMAL;
                    }

                    if column_index == 0 && selected {
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

pub(super) fn sync_video_footer_table_values(
    video_option_query: Query<(&Selectable, &VideoOptionRow, &InteractionVisualState)>,
    table_query: Query<(&ChildOf, &Children), With<VideoFooterOptionsTable>>,
    column_children_query: Query<&Children, With<Column>>,
    cell_children_query: Query<&Children, With<Cell>>,
    mut text_query: Query<(&mut TextColor, &mut TextFont, &mut Transform)>,
) {
    let mut highlighted_by_menu_footer_index: HashMap<(Entity, usize), bool> = HashMap::new();
    for (selectable, row, state) in video_option_query.iter() {
        if row.index < VIDEO_FOOTER_OPTION_START_INDEX {
            continue;
        }
        let footer_index = row.index - VIDEO_FOOTER_OPTION_START_INDEX;
        if footer_index >= VIDEO_FOOTER_OPTION_COUNT {
            continue;
        }
        highlighted_by_menu_footer_index.insert(
            (selectable.menu_entity, footer_index),
            state.selected || state.hovered || state.pressed,
        );
    }

    for (table_parent, table_children) in table_query.iter() {
        let menu_entity = table_parent.parent();
        for (column_index, column_entity) in table_children.iter().enumerate() {
            let Ok(cells) = column_children_query.get(column_entity) else {
                continue;
            };
            let Some(cell_entity) = cells.first() else {
                continue;
            };
            let Ok(cell_children) = cell_children_query.get(*cell_entity) else {
                continue;
            };

            for child in cell_children.iter() {
                let Ok((mut color, mut font, mut transform)) = text_query.get_mut(child) else {
                    continue;
                };
                let highlighted = highlighted_by_menu_footer_index
                    .get(&(menu_entity, column_index))
                    .copied()
                    .unwrap_or(false);
                if highlighted {
                    font.font_size = VIDEO_TABLE_TEXT_SELECTED_SIZE;
                    font.weight = FontWeight::BOLD;
                } else {
                    font.font_size = VIDEO_TABLE_TEXT_SIZE;
                    font.weight = FontWeight::NORMAL;
                }
                color.0 = SYSTEM_MENU_COLOR;
                transform.translation.z = VIDEO_TABLE_TEXT_Z;
                break;
            }
        }
    }
}

pub(super) fn sync_video_top_selection_bars(
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    menu_query: Query<(Entity, &SelectableMenu), With<MenuRoot>>,
    option_query: Query<(&Selectable, &VideoOptionRow), With<system_menu::SystemMenuOption>>,
    mut bar_query: Query<(&ChildOf, &mut Visibility), With<system_menu::SystemMenuSelectionBar>>,
) {
    let mut selected_top_row_by_menu: HashMap<Entity, usize> = HashMap::new();
    for (menu_entity, menu) in menu_query.iter() {
        if tabbed_focus.is_tabs_focused(menu_entity) {
            continue;
        }
        if menu.selected_index < VIDEO_TOP_OPTION_COUNT {
            selected_top_row_by_menu.insert(menu_entity, menu.selected_index);
        }
    }

    for (parent, mut visibility) in bar_query.iter_mut() {
        let Ok((selectable, row)) = option_query.get(parent.parent()) else {
            continue;
        };
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        let visible = selected_top_row_by_menu
            .get(&selectable.menu_entity)
            .is_some_and(|selected_row| *selected_row == row.index);
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn sync_video_discrete_slider_widgets(
    settings: Res<VideoSettingsState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    option_query: Query<(&Selectable, &VideoOptionRow, &InteractionVisualState)>,
    mut slider_queries: ParamSet<(
        Query<
            (
                Entity,
                &VideoOptionDiscreteSlider,
                &mut DiscreteSlider,
                &mut Visibility,
            ),
            Without<VideoOptionDiscreteSliderLabel>,
        >,
        Query<
            (
                &ChildOf,
                &mut Text2d,
                &mut TextColor,
                &mut TextFont,
                &mut Transform,
                &mut Visibility,
            ),
            (
                With<VideoOptionDiscreteSliderLabel>,
                Without<VideoOptionDiscreteSlider>,
            ),
        >,
    )>,
) {
    // Query contract:
    // - Slider-root and slider-label queries are isolated through ParamSet and
    //   complementary role markers, preventing B0001 conflicts on shared
    //   components like `Visibility`.
    let mut highlighted_by_menu_row: HashMap<(Entity, usize), bool> = HashMap::new();
    for (selectable, row, state) in option_query.iter() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        highlighted_by_menu_row.insert(
            (selectable.menu_entity, row.index),
            state.selected || state.hovered || state.pressed,
        );
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    let mut label_by_slider: HashMap<Entity, (String, bool, bool)> = HashMap::new();

    {
        let mut slider_query = slider_queries.p0();
        for (slider_entity, slider_meta, mut slider, mut visibility) in slider_query.iter_mut() {
            let active_tab =
                video_tab_kind(active_tabs.get(&slider_meta.menu_entity).copied().unwrap_or(0));
            let Some(key) = video_top_option_key(active_tab, slider_meta.row) else {
                slider.fill_color = Color::NONE;
                slider.empty_color = Color::NONE;
                slider.border_color = Color::NONE;
                slider.filled_slots = 0;
                *visibility = Visibility::Hidden;
                continue;
            };
            if !settings.initialized || !key.uses_slider() {
                slider.fill_color = Color::NONE;
                slider.empty_color = Color::NONE;
                slider.border_color = Color::NONE;
                slider.filled_slots = 0;
                *visibility = Visibility::Hidden;
                continue;
            }

            let values = key.values();
            let steps = key.slider_steps().unwrap_or_else(|| values.len()).max(1);
            let selected_index = key
                .selected_index(settings.pending)
                .min(values.len().saturating_sub(1));
            let selected = if key.slider_has_zero_state() {
                selected_index.saturating_sub(1).min(steps - 1)
            } else {
                selected_index.min(steps - 1)
            };
            let filled = key.slider_filled_slots(settings.pending).unwrap_or(0).min(steps);
            let highlighted = highlighted_by_menu_row
                .get(&(slider_meta.menu_entity, slider_meta.row))
                .copied()
                .unwrap_or(false);

            slider.steps = steps;
            slider.selected = selected;
            slider.filled_slots = filled;
            slider.slot_size = if highlighted {
                VIDEO_DISCRETE_SLIDER_SLOT_SIZE_SELECTED
            } else {
                VIDEO_DISCRETE_SLIDER_SLOT_SIZE
            };
            slider.slot_gap = VIDEO_DISCRETE_SLIDER_GAP;
            slider.fill_color = SYSTEM_MENU_COLOR;
            slider.empty_color = Color::NONE;
            slider.border_color = SYSTEM_MENU_COLOR;
            slider.border_thickness = 2.0;
            slider.fill_inset = 3.0;
            *visibility = Visibility::Visible;

            let value_label = values.get(selected_index).cloned().unwrap_or_default();
            label_by_slider.insert(slider_entity, (value_label, highlighted, true));
        }
    }

    let mut label_query = slider_queries.p1();
    for (parent, mut text, mut color, mut font, mut transform, mut visibility) in label_query.iter_mut() {
        let Some((label, highlighted, slider_visible)) = label_by_slider.get(&parent.parent()) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if !*slider_visible {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Visible;
        text.0 = format!(": {label}");
        color.0 = SYSTEM_MENU_COLOR;
        if *highlighted {
            font.font_size = VIDEO_TABLE_TEXT_SELECTED_SIZE;
            font.weight = FontWeight::BOLD;
        } else {
            font.font_size = VIDEO_TABLE_TEXT_SIZE;
            font.weight = FontWeight::NORMAL;
        }
        transform.translation.z = VIDEO_TABLE_TEXT_Z;
    }
}

pub(super) fn sync_video_option_cycler_bounds(
    settings: Res<VideoSettingsState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    mut option_query: Query<(&Selectable, &VideoOptionRow, &mut OptionCycler)>,
) {
    if !settings.initialized {
        return;
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    for (selectable, row, mut cycler) in option_query.iter_mut() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            cycler.at_min = false;
            cycler.at_max = false;
            continue;
        }

        let active_tab = video_tab_kind(active_tabs.get(&selectable.menu_entity).copied().unwrap_or(0));
        if video_top_option_uses_dropdown(active_tab, row.index) {
            cycler.at_min = true;
            cycler.at_max = true;
            continue;
        }

        let choice_count = video_top_option_choice_count(active_tab, row.index);
        if choice_count <= 1 {
            cycler.at_min = true;
            cycler.at_max = true;
            continue;
        }

        let selected_index = video_top_option_selected_index(settings.pending, active_tab, row.index)
            .unwrap_or(0)
            .min(choice_count - 1);
        cycler.at_min = selected_index == 0;
        cycler.at_max = selected_index >= choice_count - 1;
    }
}

pub(super) fn ensure_video_discrete_slider_slot_clickables(
    mut commands: Commands,
    slot_query: Query<
        (Entity, &ChildOf),
        (
            With<DiscreteSliderSlot>,
            Without<Clickable<SystemMenuActions>>,
        ),
    >,
    slider_query: Query<(Entity, &ChildOf), With<VideoOptionDiscreteSlider>>,
    option_gate_query: Query<&InteractionGate, With<MenuOptionCommand>>,
) {
    let mut gate_by_slider: HashMap<Entity, InteractionGate> = HashMap::new();
    for (slider_entity, slider_parent) in slider_query.iter() {
        let Ok(gate) = option_gate_query.get(slider_parent.parent()) else {
            continue;
        };
        gate_by_slider.insert(slider_entity, *gate);
    }

    for (slot_entity, slot_parent) in slot_query.iter() {
        let Some(gate) = gate_by_slider.get(&slot_parent.parent()).copied() else {
            continue;
        };
        commands.entity(slot_entity).insert((
            Clickable::with_region(
                vec![SystemMenuActions::Activate],
                VIDEO_DISCRETE_SLIDER_SLOT_SIZE_SELECTED + Vec2::splat(8.0),
            ),
            gate,
        ));
    }
}

pub(super) fn sync_video_cycle_arrow_positions(
    settings: Res<VideoSettingsState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    option_query: Query<(Entity, &Selectable, &VideoOptionRow), With<OptionCycler>>,
    mut arrow_query: Query<(&ChildOf, &system_menu::SystemMenuCycleArrow, &mut Transform)>,
) {
    if !settings.initialized {
        return;
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    let mut cycle_layout_by_option: HashMap<Entity, (f32, f32)> = HashMap::new();
    for (option_entity, selectable, row) in option_query.iter() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        let active_tab = video_tab_kind(active_tabs.get(&selectable.menu_entity).copied().unwrap_or(0));
        let layout = video_value_cycle_arrow_positions(video_top_option_key(active_tab, row.index));
        cycle_layout_by_option.insert(option_entity, layout);
    }

    for (parent, side, mut transform) in arrow_query.iter_mut() {
        let Some((left_x, right_x)) = cycle_layout_by_option.get(&parent.parent()).copied() else {
            continue;
        };
        transform.translation.x = match side {
            system_menu::SystemMenuCycleArrow::Left => left_x,
            system_menu::SystemMenuCycleArrow::Right => right_x,
        };
    }
}

pub(super) fn sync_video_tabs_visuals(
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    tab_root_query: Query<
        (Entity, &ChildOf, &tabs::TabBarState, &SelectableMenu),
        With<VideoTabsInteractionRoot>,
    >,
    tab_option_query: Query<(&VideoTabOption, &Selectable, &InteractionVisualState)>,
    mut tab_table_query: Query<(Entity, &ChildOf, &mut Table), With<VideoTabsTable>>,
    table_children_query: Query<&Children>,
    column_children_query: Query<&Children, With<Column>>,
    cell_children_query: Query<&Children>,
    mut cell_query: Query<&mut Cell>,
    mut text_query: Query<(&mut TextColor, &mut TextFont, &mut Transform)>,
) {
    let mut root_to_menu: HashMap<Entity, Entity> = HashMap::new();
    let mut active_tab_by_menu: HashMap<Entity, usize> = HashMap::new();
    let mut selected_tab_by_menu: HashMap<Entity, usize> = HashMap::new();
    for (tab_root_entity, parent, state, selectable_menu) in tab_root_query.iter() {
        let menu_entity = parent.parent();
        root_to_menu.insert(tab_root_entity, menu_entity);
        active_tab_by_menu.insert(menu_entity, state.active_index);
        selected_tab_by_menu.insert(menu_entity, selectable_menu.selected_index);
    }

    let mut tabs_focused_by_menu: HashMap<Entity, bool> = HashMap::new();
    for menu_entity in active_tab_by_menu.keys().copied() {
        let tabs_focused = tabbed_focus.is_tabs_focused(menu_entity);
        tabs_focused_by_menu.insert(menu_entity, tabs_focused);
    }

    let mut highlighted_tabs_by_menu: HashMap<(Entity, usize), bool> = HashMap::new();
    for (tab_option, selectable, visual_state) in tab_option_query.iter() {
        let Some(menu_entity) = root_to_menu.get(&selectable.menu_entity).copied() else {
            continue;
        };
        let tabs_focused = tabs_focused_by_menu
            .get(&menu_entity)
            .copied()
            .unwrap_or(false);
        if tabs_focused && (visual_state.hovered || visual_state.pressed) {
            highlighted_tabs_by_menu.insert((menu_entity, tab_option.index), true);
        }
    }

    let mut table_entries: Vec<(Entity, Entity, usize)> = Vec::new();
    for (table_entity, table_parent, mut table) in tab_table_query.iter_mut() {
        let menu_entity = table_parent.parent();
        let active_tab = active_tab_by_menu.get(&menu_entity).copied().unwrap_or(0);
        for (column_index, column) in table.columns.iter_mut().enumerate() {
            let sides = RectangleSides {
                top: true,
                bottom: column_index != active_tab,
                left: true,
                right: true,
            };
            if column.cell_boundary_sides != Some(sides) {
                column.cell_boundary_sides = Some(sides);
            }
        }
        table_entries.push((table_entity, menu_entity, active_tab));
    }

    for (table_entity, menu_entity, active_tab) in table_entries {
        let Ok(table_children) = table_children_query.get(table_entity) else {
            continue;
        };

        for (column_index, column_entity) in table_children.iter().enumerate() {
            let Ok(cells) = column_children_query.get(column_entity) else {
                continue;
            };
            let Some(cell_entity) = cells.first() else {
                continue;
            };
            if let Ok(mut cell) = cell_query.get_mut(*cell_entity) {
                let selected_tab = selected_tab_by_menu
                    .get(&menu_entity)
                    .copied()
                    .unwrap_or(active_tab);
                let tabs_focused = tabs_focused_by_menu
                    .get(&menu_entity)
                    .copied()
                    .unwrap_or(false);
                let highlighted = highlighted_tabs_by_menu
                    .get(&(menu_entity, column_index))
                    .copied()
                    .unwrap_or(tabs_focused && column_index == selected_tab);
                cell.set_fill_color(if highlighted {
                    SYSTEM_MENU_COLOR
                } else {
                    Color::BLACK
                });
            }
            let Ok(cell_children) = cell_children_query.get(*cell_entity) else {
                continue;
            };

            for child in cell_children.iter() {
                let Ok((mut color, mut font, mut transform)) = text_query.get_mut(child) else {
                    continue;
                };
                let selected_tab = selected_tab_by_menu
                    .get(&menu_entity)
                    .copied()
                    .unwrap_or(active_tab);
                let tabs_focused = tabs_focused_by_menu
                    .get(&menu_entity)
                    .copied()
                    .unwrap_or(false);
                let highlighted = highlighted_tabs_by_menu
                    .get(&(menu_entity, column_index))
                    .copied()
                    .unwrap_or(tabs_focused && column_index == selected_tab);
                let open = column_index == active_tab;
                if highlighted {
                    font.font_size = VIDEO_TABS_TEXT_SELECTED_SIZE;
                    font.weight = FontWeight::BOLD;
                    color.0 = Color::BLACK;
                } else {
                    font.font_size = if open {
                        VIDEO_TABS_TEXT_SELECTED_SIZE
                    } else {
                        VIDEO_TABS_TEXT_SIZE
                    };
                    font.weight = if open {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    };
                    color.0 = SYSTEM_MENU_COLOR;
                }
                transform.translation.z = VIDEO_TABLE_TEXT_Z;
                break;
            }
        }
    }
}

pub(super) fn suppress_left_cycle_arrow_for_dropdown_options(
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    option_query: Query<(Entity, &Selectable, &VideoOptionRow), With<OptionCycler>>,
    mut arrow_query: Query<(&ChildOf, &system_menu::SystemMenuCycleArrow, &mut Visibility)>,
) {
    let active_tabs = active_video_tabs_by_menu(&tab_query);
    let mut dropdown_style_options = HashSet::new();
    for (option_entity, selectable, row) in option_query.iter() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        let active_tab = video_tab_kind(active_tabs.get(&selectable.menu_entity).copied().unwrap_or(0));
        if video_top_option_uses_dropdown(active_tab, row.index) {
            dropdown_style_options.insert(option_entity);
        }
    }

    for (parent, side, mut visibility) in arrow_query.iter_mut() {
        if !dropdown_style_options.contains(&parent.parent()) {
            continue;
        }
        if *side == system_menu::SystemMenuCycleArrow::Left {
            *visibility = Visibility::Hidden;
        }
    }
}
