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
        ui::{
            discrete_slider::{DiscreteSlider, DiscreteSliderSlot},
            hover_box,
        },
    },
};

fn resolve_video_footer_highlight_by_menu(
    menu_query: &Query<(Entity, &SelectableMenu), With<MenuRoot>>,
    video_option_query: &Query<(
        &Selectable,
        &VideoOptionRow,
        &InteractionVisualState,
        Option<&InheritedVisibility>,
    )>,
) -> HashMap<Entity, usize> {
    let mut highlighted_by_menu: HashMap<Entity, (u8, usize, u64)> = HashMap::new();
    for (menu_entity, menu) in menu_query.iter() {
        if menu.selected_index < VIDEO_FOOTER_OPTION_START_INDEX
            || menu.selected_index >= VIDEO_FOOTER_OPTION_START_INDEX + VIDEO_FOOTER_OPTION_COUNT
        {
            continue;
        }
        let footer_index = menu.selected_index - VIDEO_FOOTER_OPTION_START_INDEX;
        highlighted_by_menu.insert(menu_entity, (1, footer_index, 0));
    }

    for (selectable, row, state, inherited_visibility) in video_option_query.iter() {
        if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
            continue;
        }
        if row.index < VIDEO_FOOTER_OPTION_START_INDEX {
            continue;
        }
        let footer_index = row.index - VIDEO_FOOTER_OPTION_START_INDEX;
        if footer_index >= VIDEO_FOOTER_OPTION_COUNT {
            continue;
        }
        let priority = if state.pressed {
            3
        } else if state.hovered {
            2
        } else if state.selected {
            1
        } else {
            0
        };
        if priority == 0 {
            continue;
        }
        let rank = selectable.index as u64;
        match highlighted_by_menu.get_mut(&selectable.menu_entity) {
            Some((best_priority, best_index, best_rank)) => {
                if priority > *best_priority || (priority == *best_priority && rank >= *best_rank) {
                    *best_priority = priority;
                    *best_index = footer_index;
                    *best_rank = rank;
                }
            }
            None => {
                highlighted_by_menu.insert(selectable.menu_entity, (priority, footer_index, rank));
            }
        }
    }

    highlighted_by_menu
        .into_iter()
        .map(|(menu_entity, (_, footer_index, _))| (menu_entity, footer_index))
        .collect()
}

pub(super) fn sync_video_top_table_values(
    settings: Res<VideoSettingsState>,
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    menu_query: Query<(Entity, &SelectableMenu), With<MenuRoot>>,
    video_option_query: Query<(
        &Selectable,
        &VideoOptionRow,
        &InteractionVisualState,
        Option<&InheritedVisibility>,
    )>,
    table_query: Query<(&ChildOf, &Children), With<VideoTopOptionsTable>>,
    scroll_content_query: Query<&ChildOf, With<VideoTopOptionsScrollContent>>,
    scroll_root_query: Query<
        &crate::systems::ui::scroll::ScrollableTableAdapter,
        With<VideoTopOptionsScrollRoot>,
    >,
    column_children_query: Query<&Children, With<Column>>,
    cell_children_query: Query<&Children, With<Cell>>,
    mut text_query: Query<(&mut Text2d, &mut TextColor, &mut TextFont, &mut Transform)>,
) {
    if !settings.initialized {
        return;
    }

    let mut selected_top_row_by_menu: HashMap<Entity, usize> = HashMap::new();
    for (menu_entity, menu) in menu_query.iter() {
        if tabbed_focus.is_tabs_focused(menu_entity) {
            continue;
        }
        if menu.selected_index < VIDEO_TOP_OPTION_COUNT {
            selected_top_row_by_menu.insert(menu_entity, menu.selected_index);
        }
    }

    let mut pressed_by_menu_row: HashMap<(Entity, usize), bool> = HashMap::new();
    for (selectable, row, state, inherited_visibility) in video_option_query.iter() {
        if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
            continue;
        }
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        if state.pressed {
            pressed_by_menu_row.insert((selectable.menu_entity, row.index), true);
        }
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    for (table_parent, table_children) in table_query.iter() {
        let parent_entity = table_parent.parent();
        let menu_entity = if active_tabs.contains_key(&parent_entity) {
            parent_entity
        } else if let Ok(content_parent) = scroll_content_query.get(parent_entity) {
            let root_entity = content_parent.parent();
            if let Ok(adapter) = scroll_root_query.get(root_entity) {
                adapter.owner
            } else {
                continue;
            }
        } else {
            continue;
        };
        let Some(active_tab) = active_tabs.get(&menu_entity).copied().map(video_tab_kind) else {
            continue;
        };
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

                    let selected = pressed_by_menu_row
                        .get(&(menu_entity, row_index))
                        .copied()
                        .unwrap_or(false)
                        || selected_top_row_by_menu
                            .get(&menu_entity)
                            .is_some_and(|selected_row| *selected_row == row_index);
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
    menu_query: Query<(Entity, &SelectableMenu), With<MenuRoot>>,
    video_option_query: Query<(
        &Selectable,
        &VideoOptionRow,
        &InteractionVisualState,
        Option<&InheritedVisibility>,
    )>,
    table_query: Query<(&ChildOf, &Children), With<VideoFooterOptionsTable>>,
    column_children_query: Query<&Children, With<Column>>,
    cell_children_query: Query<&Children, With<Cell>>,
    mut text_query: Query<(&mut TextColor, &mut TextFont, &mut Transform)>,
) {
    let highlighted_by_menu =
        resolve_video_footer_highlight_by_menu(&menu_query, &video_option_query);

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
                let highlighted = highlighted_by_menu
                    .get(&menu_entity)
                    .is_some_and(|highlighted_index| *highlighted_index == column_index);
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

pub(super) fn sync_video_footer_selection_indicators(
    menu_query: Query<(Entity, &SelectableMenu), With<MenuRoot>>,
    video_option_query: Query<(
        &Selectable,
        &VideoOptionRow,
        &InteractionVisualState,
        Option<&InheritedVisibility>,
    )>,
    option_query: Query<(&Selectable, &VideoOptionRow), With<system_menu::SystemMenuOption>>,
    mut indicator_query: Query<
        (&ChildOf, &mut Visibility),
        With<system_menu::SystemMenuSelectionIndicator>,
    >,
) {
    let highlighted_by_menu =
        resolve_video_footer_highlight_by_menu(&menu_query, &video_option_query);

    for (parent, mut visibility) in indicator_query.iter_mut() {
        let Ok((selectable, row)) = option_query.get(parent.parent()) else {
            continue;
        };
        if row.index < VIDEO_FOOTER_OPTION_START_INDEX
            || row.index >= VIDEO_FOOTER_OPTION_START_INDEX + VIDEO_FOOTER_OPTION_COUNT
        {
            continue;
        }
        let footer_index = row.index - VIDEO_FOOTER_OPTION_START_INDEX;
        let highlighted = highlighted_by_menu
            .get(&selectable.menu_entity)
            .is_some_and(|highlighted_index| *highlighted_index == footer_index);
        *visibility = if highlighted {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
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
            let Some(active_tab) = active_tabs
                .get(&slider_meta.menu_entity)
                .copied()
                .map(video_tab_kind)
            else {
                slider.fill_color = Color::NONE;
                slider.empty_color = Color::NONE;
                slider.border_color = Color::NONE;
                slider.filled_slots = 0;
                *visibility = Visibility::Hidden;
                continue;
            };
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
            let filled = key
                .slider_filled_slots(settings.pending)
                .unwrap_or(0)
                .min(steps);
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
    for (parent, mut text, mut color, mut font, mut transform, mut visibility) in
        label_query.iter_mut()
    {
        let Some((label, highlighted, slider_visible)) = label_by_slider.get(&parent.parent())
        else {
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

        let Some(active_tab) = active_tabs
            .get(&selectable.menu_entity)
            .copied()
            .map(video_tab_kind)
        else {
            cycler.at_min = true;
            cycler.at_max = true;
            continue;
        };
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

        let selected_index =
            video_top_option_selected_index(settings.pending, active_tab, row.index)
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
        let Some(active_tab) = active_tabs
            .get(&selectable.menu_entity)
            .copied()
            .map(video_tab_kind)
        else {
            continue;
        };
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
        let Some(active_tab) = active_tab_by_menu.get(&menu_entity).copied() else {
            continue;
        };
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
    mut arrow_query: Query<(
        &ChildOf,
        &system_menu::SystemMenuCycleArrow,
        &mut Visibility,
    )>,
) {
    let active_tabs = active_video_tabs_by_menu(&tab_query);
    let mut dropdown_style_options = HashSet::new();
    for (option_entity, selectable, row) in option_query.iter() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        let Some(active_tab) = active_tabs
            .get(&selectable.menu_entity)
            .copied()
            .map(video_tab_kind)
        else {
            continue;
        };
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

#[inline]
fn write_hover_box_content(content: &mut hover_box::HoverBoxContent, text: &str) {
    if content.text == text {
        return;
    }
    content.text.clear();
    content.text.push_str(text);
}

pub(super) fn sync_video_top_option_hover_descriptions(
    dropdown_state: Res<DropdownLayerState>,
    dropdown_anchor_state: Res<DropdownAnchorState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    menu_query: Query<(Entity, &MenuStack, &SelectableMenu), With<MenuRoot>>,
    mut option_query: Query<
        (
            &Selectable,
            &VideoOptionRow,
            &mut hover_box::HoverBoxContent,
        ),
        (
            Without<VideoResolutionDropdownItem>,
            Without<VideoResolutionDropdown>,
        ),
    >,
    dropdown_query: Query<
        (Entity, &ChildOf, &Visibility),
        (
            With<VideoResolutionDropdown>,
            Without<VideoResolutionDropdownItem>,
        ),
    >,
    mut dropdown_item_query: Query<
        (
            &ChildOf,
            &VideoResolutionDropdownItem,
            &mut hover_box::HoverBoxContent,
        ),
        (With<VideoResolutionDropdownItem>, Without<VideoOptionRow>),
    >,
) {
    let active_tabs = active_video_tabs_by_menu(&tab_query);

    for (selectable, row, mut content) in option_query.iter_mut() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            write_hover_box_content(&mut content, "");
            continue;
        }
        let description = active_tabs
            .get(&selectable.menu_entity)
            .copied()
            .map(video_tab_kind)
            .and_then(|active_tab| video_top_option_key(active_tab, row.index))
            .map(VideoTopOptionKey::description)
            .unwrap_or_default();
        write_hover_box_content(&mut content, description);
    }

    let mut open_dropdown_key_by_menu: HashMap<Entity, VideoTopOptionKey> = HashMap::new();
    for (_, open_parent) in dropdown_state.open_parents_snapshot() {
        let Ok((menu_entity, menu_stack, selectable_menu)) = menu_query.get(open_parent) else {
            continue;
        };
        if menu_entity != open_parent || menu_stack.current_page() != Some(MenuPage::Video) {
            continue;
        }
        let row = dropdown_anchor_state.row_for_parent(
            open_parent,
            open_parent,
            selectable_menu.selected_index,
        );
        if row >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        let Some(active_tab) = active_tabs.get(&open_parent).copied().map(video_tab_kind) else {
            continue;
        };
        let Some(key) = video_top_option_key(active_tab, row) else {
            continue;
        };
        if key.uses_dropdown() {
            open_dropdown_key_by_menu.insert(open_parent, key);
        }
    }

    let dropdown_parent_by_entity: HashMap<Entity, Entity> = dropdown_query
        .iter()
        .filter_map(|(dropdown_entity, parent, visibility)| {
            (*visibility == Visibility::Visible).then_some((dropdown_entity, parent.parent()))
        })
        .collect();

    for (parent, item, mut content) in dropdown_item_query.iter_mut() {
        let Some(menu_entity) = dropdown_parent_by_entity.get(&parent.parent()).copied() else {
            write_hover_box_content(&mut content, "");
            continue;
        };
        let description = open_dropdown_key_by_menu
            .get(&menu_entity)
            .copied()
            .and_then(|key| key.value_description(item.index))
            .unwrap_or_default();
        write_hover_box_content(&mut content, description);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        ecs::system::{IntoSystem, SystemState},
        sprite::Anchor,
    };

    use crate::entities::text::{Cell, Column, Row, Table, TextContent};
    use crate::systems::ui::{
        dropdown,
        layer::{UiLayer, UiLayerKind},
        tabs::{TabBar, TabBarState},
    };

    fn test_menu_root(gate: InteractionGate) -> MenuRoot {
        MenuRoot {
            host: MenuHost::Pause,
            gate,
        }
    }

    fn test_selectable_menu(selected_index: usize) -> SelectableMenu {
        SelectableMenu::new(
            selected_index,
            vec![KeyCode::ArrowUp],
            vec![KeyCode::ArrowDown],
            vec![KeyCode::Enter],
            true,
        )
    }

    #[test]
    fn footer_highlight_resolver_prefers_pressed_then_hovered_then_selected() {
        let mut world = World::new();
        let menu_entity = world
            .spawn((
                test_menu_root(InteractionGate::PauseMenuOnly),
                test_selectable_menu(VIDEO_FOOTER_OPTION_START_INDEX),
            ))
            .id();

        world.spawn((
            Selectable::new(menu_entity, VIDEO_FOOTER_OPTION_START_INDEX),
            VideoOptionRow {
                index: VIDEO_FOOTER_OPTION_START_INDEX,
            },
            InteractionVisualState {
                selected: true,
                ..default()
            },
        ));
        world.spawn((
            Selectable::new(menu_entity, VIDEO_FOOTER_OPTION_START_INDEX + 1),
            VideoOptionRow {
                index: VIDEO_FOOTER_OPTION_START_INDEX + 1,
            },
            InteractionVisualState {
                hovered: true,
                ..default()
            },
        ));
        world.spawn((
            Selectable::new(menu_entity, VIDEO_FOOTER_OPTION_START_INDEX + 2),
            VideoOptionRow {
                index: VIDEO_FOOTER_OPTION_START_INDEX + 2,
            },
            InteractionVisualState {
                pressed: true,
                ..default()
            },
        ));

        let mut state: SystemState<(
            Query<(Entity, &SelectableMenu), With<MenuRoot>>,
            Query<(
                &Selectable,
                &VideoOptionRow,
                &InteractionVisualState,
                Option<&InheritedVisibility>,
            )>,
        )> = SystemState::new(&mut world);
        let (menu_query, option_query) = state.get(&world);
        let highlighted = resolve_video_footer_highlight_by_menu(&menu_query, &option_query);
        assert_eq!(highlighted.get(&menu_entity).copied(), Some(2));
    }

    #[test]
    fn footer_highlight_resolver_breaks_ties_by_higher_selectable_index() {
        let mut world = World::new();
        let menu_entity = world
            .spawn((
                test_menu_root(InteractionGate::PauseMenuOnly),
                test_selectable_menu(VIDEO_FOOTER_OPTION_START_INDEX),
            ))
            .id();

        world.spawn((
            Selectable::new(menu_entity, VIDEO_FOOTER_OPTION_START_INDEX),
            VideoOptionRow {
                index: VIDEO_FOOTER_OPTION_START_INDEX,
            },
            InteractionVisualState {
                hovered: true,
                ..default()
            },
        ));
        world.spawn((
            Selectable::new(menu_entity, VIDEO_FOOTER_OPTION_START_INDEX + 1),
            VideoOptionRow {
                index: VIDEO_FOOTER_OPTION_START_INDEX + 1,
            },
            InteractionVisualState {
                hovered: true,
                ..default()
            },
        ));

        let mut state: SystemState<(
            Query<(Entity, &SelectableMenu), With<MenuRoot>>,
            Query<(
                &Selectable,
                &VideoOptionRow,
                &InteractionVisualState,
                Option<&InheritedVisibility>,
            )>,
        )> = SystemState::new(&mut world);
        let (menu_query, option_query) = state.get(&world);
        let highlighted = resolve_video_footer_highlight_by_menu(&menu_query, &option_query);
        assert_eq!(highlighted.get(&menu_entity).copied(), Some(1));
    }

    #[test]
    fn hover_description_sync_populates_option_and_open_dropdown_value_content() {
        let mut app = App::new();
        app.add_systems(Update, sync_video_top_option_hover_descriptions);

        let menu_entity = app
            .world_mut()
            .spawn((
                test_menu_root(InteractionGate::PauseMenuOnly),
                MenuStack::new(MenuPage::Video),
                test_selectable_menu(0),
            ))
            .id();

        app.world_mut().spawn((
            TabBar::new(menu_entity),
            TabBarState { active_index: 2 }, // Advanced
            tabbed_menu::TabbedMenuConfig::new(
                VIDEO_TOP_OPTION_COUNT,
                VIDEO_FOOTER_OPTION_START_INDEX,
                VIDEO_FOOTER_OPTION_COUNT,
            ),
        ));

        let option_entity = app
            .world_mut()
            .spawn((
                Selectable::new(menu_entity, 0),
                VideoOptionRow { index: 0 }, // Tonemapping in Advanced tab
                hover_box::HoverBoxContent::default(),
            ))
            .id();

        let dropdown_entity = app
            .world_mut()
            .spawn((
                VideoResolutionDropdown,
                UiLayer::new(menu_entity, UiLayerKind::Dropdown),
                Visibility::Hidden,
                test_selectable_menu(1),
            ))
            .id();
        app.world_mut()
            .entity_mut(menu_entity)
            .add_child(dropdown_entity);

        let dropdown_item = app
            .world_mut()
            .spawn((
                VideoResolutionDropdownItem { index: 1 },
                hover_box::HoverBoxContent::default(),
            ))
            .id();
        app.world_mut()
            .entity_mut(dropdown_entity)
            .add_child(dropdown_item);

        let mut dropdown_state = DropdownLayerState::default();
        {
            let world = app.world_mut();
            let mut query_state: SystemState<(
                Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
                Query<&mut SelectableMenu, With<VideoResolutionDropdown>>,
            )> = SystemState::new(world);
            let (mut dropdown_query, mut dropdown_menu_query) = query_state.get_mut(world);
            dropdown::open_for_parent::<VideoResolutionDropdown>(
                menu_entity,
                menu_entity,
                1,
                &mut dropdown_state,
                &mut dropdown_query,
                &mut dropdown_menu_query,
            );
            query_state.apply(world);
        }
        app.insert_resource(dropdown_state);
        let mut dropdown_anchor_state = DropdownAnchorState::default();
        dropdown_anchor_state.set_for_parent(menu_entity, menu_entity, 0);
        app.insert_resource(dropdown_anchor_state);

        app.update();

        let option_content = app
            .world()
            .get::<hover_box::HoverBoxContent>(option_entity)
            .expect("option hover box content");
        assert_eq!(
            option_content.text,
            VideoTopOptionKey::Tonemapping.description()
        );

        let dropdown_content = app
            .world()
            .get::<hover_box::HoverBoxContent>(dropdown_item)
            .expect("dropdown hover box content");
        assert_eq!(
            dropdown_content.text,
            VideoTopOptionKey::Tonemapping
                .value_description(1)
                .expect("tonemapping index description")
        );
    }

    #[test]
    fn top_table_sync_resolves_menu_owner_from_scroll_content_parent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, sync_video_top_table_values);
        app.init_resource::<tabbed_menu::TabbedMenuFocusState>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();

        let mut settings = VideoSettingsState::default();
        settings.initialized = true;
        app.insert_resource(settings);

        let menu_entity = app
            .world_mut()
            .spawn((
                test_menu_root(InteractionGate::PauseMenuOnly),
                test_selectable_menu(0),
            ))
            .id();
        app.world_mut().spawn((
            TabBar::new(menu_entity),
            TabBarState { active_index: 0 }, // Display
            tabbed_menu::TabbedMenuConfig::new(
                VIDEO_TOP_OPTION_COUNT,
                VIDEO_FOOTER_OPTION_START_INDEX,
                VIDEO_FOOTER_OPTION_COUNT,
            ),
        ));

        let scroll_root = app
            .world_mut()
            .spawn((
                VideoTopOptionsScrollRoot,
                crate::systems::ui::scroll::ScrollableTableAdapter::new(
                    menu_entity,
                    VIDEO_TOP_OPTION_COUNT,
                    40.0,
                    0.0,
                ),
            ))
            .id();
        let scroll_content = app.world_mut().spawn(VideoTopOptionsScrollContent).id();
        app.world_mut()
            .entity_mut(scroll_root)
            .add_child(scroll_content);

        let table = Table {
            columns: vec![
                Column::new(
                    vec![Cell::new(TextContent::new(
                        "name_placeholder".to_string(),
                        SYSTEM_MENU_COLOR,
                        VIDEO_TABLE_TEXT_SIZE,
                    ))],
                    220.0,
                    Vec2::ZERO,
                    Anchor::CENTER,
                    false,
                ),
                Column::new(
                    vec![Cell::new(TextContent::new(
                        "value_placeholder".to_string(),
                        SYSTEM_MENU_COLOR,
                        VIDEO_TABLE_TEXT_SIZE,
                    ))],
                    220.0,
                    Vec2::ZERO,
                    Anchor::CENTER,
                    false,
                ),
            ],
            rows: vec![Row { height: 40.0 }],
        };

        let table_entity = app.world_mut().spawn((VideoTopOptionsTable, table)).id();
        app.world_mut()
            .entity_mut(scroll_content)
            .add_child(table_entity);

        // First update materializes table/column/cell hook-spawned children.
        app.update();
        // Second update applies sync_video_top_table_values to spawned text nodes.
        app.update();

        let world = app.world();
        let table_children = world
            .get::<Children>(table_entity)
            .expect("table columns spawned");
        assert_eq!(table_children.len(), 2);

        let name_column = table_children[0];
        let value_column = table_children[1];
        let name_cell = *world
            .get::<Children>(name_column)
            .expect("name column cells")
            .first()
            .expect("first name row");
        let value_cell = *world
            .get::<Children>(value_column)
            .expect("value column cells")
            .first()
            .expect("first value row");

        let name_text_entity = world
            .get::<Children>(name_cell)
            .expect("name cell children")
            .iter()
            .find(|entity| world.get::<Text2d>(*entity).is_some())
            .expect("name text child");
        let value_text_entity = world
            .get::<Children>(value_cell)
            .expect("value cell children")
            .iter()
            .find(|entity| world.get::<Text2d>(*entity).is_some())
            .expect("value text child");

        let expected_label = video_top_option_labels(VideoTabKind::Display)[0];
        let expected_value = video_top_value_strings(
            app.world().resource::<VideoSettingsState>().pending,
            VideoTabKind::Display,
        )[0]
        .clone();

        let name_text = world
            .get::<Text2d>(name_text_entity)
            .expect("name text value");
        let value_text = world
            .get::<Text2d>(value_text_entity)
            .expect("value text value");
        assert_eq!(name_text.0, expected_label);
        assert_eq!(value_text.0, expected_value);

        let name_color = world
            .get::<TextColor>(name_text_entity)
            .expect("name text color")
            .0;
        assert_eq!(name_color, Color::srgb(0.0, 0.08, 0.0));
    }

    #[test]
    fn video_visual_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut top_table_system = IntoSystem::into_system(sync_video_top_table_values);
        top_table_system.initialize(&mut world);

        let mut footer_table_system = IntoSystem::into_system(sync_video_footer_table_values);
        footer_table_system.initialize(&mut world);

        let mut footer_indicators_system =
            IntoSystem::into_system(sync_video_footer_selection_indicators);
        footer_indicators_system.initialize(&mut world);

        let mut top_bars_system = IntoSystem::into_system(sync_video_top_selection_bars);
        top_bars_system.initialize(&mut world);

        let mut slider_widgets_system = IntoSystem::into_system(sync_video_discrete_slider_widgets);
        slider_widgets_system.initialize(&mut world);

        let mut cycler_bounds_system = IntoSystem::into_system(sync_video_option_cycler_bounds);
        cycler_bounds_system.initialize(&mut world);

        let mut slider_clickables_system =
            IntoSystem::into_system(ensure_video_discrete_slider_slot_clickables);
        slider_clickables_system.initialize(&mut world);

        let mut cycle_arrows_system = IntoSystem::into_system(sync_video_cycle_arrow_positions);
        cycle_arrows_system.initialize(&mut world);

        let mut tabs_visuals_system = IntoSystem::into_system(sync_video_tabs_visuals);
        tabs_visuals_system.initialize(&mut world);

        let mut suppress_left_arrow_system =
            IntoSystem::into_system(suppress_left_cycle_arrow_for_dropdown_options);
        suppress_left_arrow_system.initialize(&mut world);

        let mut hover_descriptions_system =
            IntoSystem::into_system(sync_video_top_option_hover_descriptions);
        hover_descriptions_system.initialize(&mut world);
    }
}
