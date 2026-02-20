use super::*;
use bevy::sprite::Anchor;
use bevy::text::{TextBounds, TextLayoutInfo};

use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{centered_text_y_correction, Table},
    },
    systems::colors::SYSTEM_MENU_COLOR,
};

fn dropdown_anchor_in_menu_space(
    menu_entity: Entity,
    row: usize,
    dropdown_rows: usize,
    scroll_offset_by_menu: &HashMap<Entity, f32>,
    table_query: &Query<
        (&ChildOf, &Table, &GlobalTransform),
        (With<VideoTopOptionsTable>, Without<VideoResolutionDropdown>),
    >,
) -> Vec2 {
    if let Some(offset_px) = scroll_offset_by_menu.get(&menu_entity).copied() {
        let dropdown_height = dropdown_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
        let row_top_y = video_top_row_top_y(row) + offset_px;
        return Vec2::new(
            VIDEO_VALUE_COLUMN_CENTER_X,
            row_top_y - dropdown_height * 0.5,
        );
    }

    for (parent, table, table_transform) in table_query.iter() {
        if parent.parent() != menu_entity {
            continue;
        }
        if table.columns.len() < 2 {
            break;
        }

        let table_translation = table_transform.translation();
        let value_center_x =
            table_translation.x + table.columns[0].width + table.columns[1].width * 0.5;
        let row_top_y = table_translation.y
            - table
                .rows
                .iter()
                .take(row)
                .map(|row| row.height.max(1.0))
                .sum::<f32>();
        let dropdown_height = dropdown_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
        let center_y = row_top_y - dropdown_height * 0.5;
        return Vec2::new(value_center_x, center_y);
    }

    Vec2::new(
        VIDEO_VALUE_COLUMN_CENTER_X,
        dropdown_center_y_from_row_top(row, dropdown_rows),
    )
}

pub(super) fn sync_resolution_dropdown_items(
    settings: Res<VideoSettingsState>,
    dropdown_state: Res<DropdownLayerState>,
    dropdown_anchor_state: Res<DropdownAnchorState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    menu_query: Query<(Entity, &MenuStack, &SelectableMenu), With<MenuRoot>>,
    scroll_root_query: Query<
        (
            &crate::systems::ui::scroll::ScrollableTableAdapter,
            &crate::systems::ui::scroll::ScrollState,
        ),
        With<VideoTopOptionsScrollRoot>,
    >,
    table_query: Query<
        (&ChildOf, &Table, &GlobalTransform),
        (With<VideoTopOptionsTable>, Without<VideoResolutionDropdown>),
    >,
    mut dropdown_query: Query<
        (Entity, &ChildOf, &mut Sprite, &mut Transform, &Visibility),
        (
            With<VideoResolutionDropdown>,
            Without<VideoResolutionDropdownItem>,
            Without<VideoTopOptionsTable>,
        ),
    >,
    mut border_query: Query<(&ChildOf, &mut HollowRectangle), With<VideoResolutionDropdownBorder>>,
    mut item_query: Query<
        (
            &ChildOf,
            &VideoResolutionDropdownItem,
            &mut Selectable,
            &mut VideoResolutionDropdownItemBaseY,
            &InteractionVisualState,
            &mut Text2d,
            &mut TextColor,
            &mut TextFont,
            &mut Clickable<SystemMenuActions>,
            &mut Visibility,
        ),
        (
            With<VideoResolutionDropdownItem>,
            Without<VideoResolutionDropdown>,
        ),
    >,
) {
    // Query contract:
    // - `dropdown_query` only mutates dropdown container geometry/visibility.
    // - `item_query` only mutates dropdown row entities (text/selectable/clickable/visibility).
    // - `table_query` and scroll/menu/tab queries are read-only lookup sources.
    // This keeps mutable access disjoint between parent dropdown surfaces and row items.
    if !settings.initialized {
        return;
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    let mut open_context_by_menu: HashMap<Entity, (usize, Vec<String>, usize)> = HashMap::new();
    let scroll_offset_by_menu: HashMap<Entity, f32> = scroll_root_query
        .iter()
        .map(|(adapter, state)| (adapter.owner, state.offset_px))
        .collect();
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
        let values = video_top_option_values(active_tab, row);
        if values.is_empty() {
            continue;
        }
        let selected_index = video_top_option_selected_index(settings.pending, active_tab, row)
            .unwrap_or(0)
            .min(values.len().saturating_sub(1));
        open_context_by_menu.insert(open_parent, (row, values, selected_index));
    }
    let mut open_dropdown_to_menu: HashMap<Entity, Entity> = HashMap::new();

    for (dropdown_entity, parent, mut sprite, mut transform, _) in dropdown_query.iter_mut() {
        let menu_entity = parent.parent();
        let Some((row, values, _)) = open_context_by_menu.get(&menu_entity) else {
            continue;
        };
        let dropdown_rows = values.len().max(1);
        let dropdown_height = dropdown_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
        let anchor = dropdown_anchor_in_menu_space(
            menu_entity,
            *row,
            dropdown_rows,
            &scroll_offset_by_menu,
            &table_query,
        );
        sprite.custom_size = Some(Vec2::new(
            VIDEO_RESOLUTION_DROPDOWN_WIDTH + VIDEO_RESOLUTION_DROPDOWN_BACKGROUND_PAD_X * 2.0,
            dropdown_height,
        ));
        transform.translation.x = anchor.x;
        transform.translation.y = anchor.y;
        transform.translation.z = VIDEO_RESOLUTION_DROPDOWN_Z;
        open_dropdown_to_menu.insert(dropdown_entity, menu_entity);
    }

    for (parent, mut border) in border_query.iter_mut() {
        let Some(menu_entity) = open_dropdown_to_menu.get(&parent.parent()) else {
            continue;
        };
        let Some((_, values, _)) = open_context_by_menu.get(menu_entity) else {
            continue;
        };
        let dropdown_rows = values.len().max(1);
        let dropdown_height = dropdown_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
        border.dimensions = Vec2::new(
            VIDEO_RESOLUTION_DROPDOWN_WIDTH + VIDEO_RESOLUTION_DROPDOWN_BACKGROUND_PAD_X * 2.0
                - 6.0,
            dropdown_height - 6.0,
        );
    }

    for (
        parent,
        item,
        mut selectable,
        mut base_y,
        state,
        mut text,
        mut color,
        mut font,
        mut clickable,
        mut visibility,
    ) in item_query.iter_mut()
    {
        let Some(menu_entity) = open_dropdown_to_menu.get(&parent.parent()) else {
            text.0.clear();
            clickable.region = None;
            *visibility = Visibility::Hidden;
            selectable.menu_entity = Entity::PLACEHOLDER;
            continue;
        };
        let Some((_, open_values, open_selected_index)) = open_context_by_menu.get(menu_entity)
        else {
            text.0.clear();
            clickable.region = None;
            *visibility = Visibility::Hidden;
            selectable.menu_entity = Entity::PLACEHOLDER;
            continue;
        };
        if item.index >= open_values.len() {
            text.0.clear();
            clickable.region = None;
            *visibility = Visibility::Hidden;
            selectable.menu_entity = Entity::PLACEHOLDER;
            continue;
        }

        text.0 = open_values[item.index].clone();
        selectable.menu_entity = parent.parent();
        clickable.region = Some(Vec2::new(
            VIDEO_RESOLUTION_DROPDOWN_WIDTH - 8.0,
            VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT,
        ));
        *visibility = Visibility::Visible;
        base_y.0 = dropdown_item_local_center_y(item.index, open_values.len());

        let focused = state.selected || state.hovered || state.pressed;
        if focused {
            color.0 = SYSTEM_MENU_COLOR;
            font.font_size = VIDEO_TABLE_TEXT_SELECTED_SIZE;
            font.weight = FontWeight::BOLD;
        } else if item.index == *open_selected_index {
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

pub(super) fn ensure_resolution_dropdown_value_arrows(
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

pub(super) fn update_resolution_dropdown_value_arrows(
    settings: Res<VideoSettingsState>,
    dropdown_state: Res<DropdownLayerState>,
    dropdown_anchor_state: Res<DropdownAnchorState>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    menu_query: Query<(Entity, &MenuStack, &SelectableMenu), With<MenuRoot>>,
    dropdown_parent_query: Query<(Entity, &ChildOf), With<VideoResolutionDropdown>>,
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
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Query contract:
    // - `arrow_query` mutates arrow entities only.
    // - `dropdown_parent_query` and menu/tab/dropdown state are read-only.
    // - material mutation is isolated to `Assets<ColorMaterial>` writes.
    // This avoids aliasing mutable component access across dropdown rows/containers.
    if !settings.initialized {
        return;
    }

    let active_tabs = active_video_tabs_by_menu(&tab_query);
    let dropdown_entity_by_parent: HashMap<Entity, Entity> = dropdown_parent_query
        .iter()
        .map(|(dropdown_entity, parent)| (parent.parent(), dropdown_entity))
        .collect();
    let mut open_context_by_dropdown: HashMap<Entity, (usize, usize)> = HashMap::new();
    for (_, open_parent) in dropdown_state.open_parents_snapshot() {
        let Some(dropdown_entity) = dropdown_entity_by_parent.get(&open_parent).copied() else {
            continue;
        };
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
        let values = video_top_option_values(active_tab, row);
        if values.is_empty() {
            continue;
        }
        let selected_index = video_top_option_selected_index(settings.pending, active_tab, row)
            .unwrap_or(0)
            .min(values.len().saturating_sub(1));
        open_context_by_dropdown.insert(dropdown_entity, (selected_index, values.len()));
    }

    for (parent, side, mut visibility, mut transform, material_handle) in arrow_query.iter_mut() {
        let Some((selected_index, value_count)) =
            open_context_by_dropdown.get(&parent.parent()).copied()
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let selected_y = dropdown_item_local_center_y(selected_index, value_count);
        transform.translation.x = match side {
            VideoResolutionDropdownValueArrowSide::Left => -VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD,
            VideoResolutionDropdownValueArrowSide::Right => VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD,
        };
        transform.translation.y = selected_y;
        transform.translation.z = VIDEO_RESOLUTION_DROPDOWN_ARROW_Z;

        *visibility = Visibility::Visible;

        if let Some(material) = materials.get_mut(material_handle.0.id()) {
            material.color = SYSTEM_MENU_COLOR;
        }
    }
}

pub(super) fn recenter_resolution_dropdown_item_text(
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
    // Query contract:
    // - single-query mutation over dropdown item text transforms only.
    // - no overlapping mutable text queries in this system.
    for (base_y, anchor, bounds, text_layout, mut transform) in item_query.iter_mut() {
        let center_correction = centered_text_y_correction(anchor, bounds, text_layout);
        let target_y = base_y.0 + center_correction;
        if (transform.translation.y - target_y).abs() > 0.001 {
            transform.translation.y = target_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::IntoSystem;

    use super::*;

    #[test]
    fn dropdown_view_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut sync_items_system = IntoSystem::into_system(sync_resolution_dropdown_items);
        sync_items_system.initialize(&mut world);

        let mut ensure_arrows_system =
            IntoSystem::into_system(ensure_resolution_dropdown_value_arrows);
        ensure_arrows_system.initialize(&mut world);

        let mut update_arrows_system =
            IntoSystem::into_system(update_resolution_dropdown_value_arrows);
        update_arrows_system.initialize(&mut world);

        let mut recenter_text_system =
            IntoSystem::into_system(recenter_resolution_dropdown_item_text);
        recenter_text_system.initialize(&mut world);
    }
}
