use std::collections::HashMap;

use bevy::prelude::*;

use super::*;
use crate::systems::{
    interaction::InteractionVisualState,
    ui::scroll::{ScrollFocusFollowLock, ScrollState},
};
const VISIBILITY_EPSILON: f32 = 0.001;

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct ScrollableTableAdapter {
    pub menu_entity: Entity,
    pub row_count: usize,
    pub row_extent: f32,
    pub leading_padding: f32,
}

impl ScrollableTableAdapter {
    pub const fn new(
        menu_entity: Entity,
        row_count: usize,
        row_extent: f32,
        leading_padding: f32,
    ) -> Self {
        Self {
            menu_entity,
            row_count,
            row_extent,
            leading_padding,
        }
    }
}

pub(super) fn sync_video_top_scroll_focus_follow(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    menu_query: Query<&SelectableMenu, With<MenuRoot>>,
    option_query: Query<(&Selectable, &VideoOptionRow, &InteractionVisualState), With<MenuOptionCommand>>,
    mut root_query: Query<
        (
            &ScrollableTableAdapter,
            &mut ScrollState,
            &mut ScrollFocusFollowLock,
        ),
        With<VideoTopOptionsScrollRoot>,
    >,
) {
    let keyboard_navigation = keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::PageUp)
        || keyboard_input.just_pressed(KeyCode::PageDown)
        || keyboard_input.just_pressed(KeyCode::Home)
        || keyboard_input.just_pressed(KeyCode::End);
    let mut keyboard_locked_selected_by_menu_row = HashMap::new();
    for (selectable, row, visual_state) in option_query.iter() {
        if row.index >= VIDEO_TOP_OPTION_COUNT || !visual_state.selected {
            continue;
        }
        keyboard_locked_selected_by_menu_row.insert(
            (selectable.menu_entity, row.index),
            visual_state.keyboard_locked,
        );
    }

    for (adapter, mut state, mut focus_lock) in root_query.iter_mut() {
        if tabbed_focus.is_tabs_focused(adapter.menu_entity) {
            continue;
        }
        let Ok(menu) = menu_query.get(adapter.menu_entity) else {
            continue;
        };
        if menu.selected_index >= adapter.row_count {
            continue;
        }
        if keyboard_navigation {
            focus_lock.manual_override = false;
        }
        if focus_lock.manual_override {
            continue;
        }
        let selected_is_keyboard_locked = keyboard_locked_selected_by_menu_row
            .get(&(adapter.menu_entity, menu.selected_index))
            .copied()
            .unwrap_or(false);
        if !keyboard_navigation && !selected_is_keyboard_locked {
            continue;
        }
        focus_scroll_offset_to_row(
            &mut state,
            menu.selected_index,
            adapter.row_extent,
            adapter.leading_padding,
        );
    }
}

pub(super) fn sync_video_top_option_hit_regions_to_viewport(
    root_query: Query<(&ScrollableTableAdapter, &ScrollState), With<VideoTopOptionsScrollRoot>>,
    mut option_query: Query<
        (&Selectable, &VideoOptionRow, &mut Clickable<SystemMenuActions>),
        With<MenuOptionCommand>,
    >,
) {
    let mut adapter_state_by_menu = HashMap::new();
    for (adapter, state) in root_query.iter() {
        adapter_state_by_menu.insert(adapter.menu_entity, (*adapter, *state));
    }

    for (selectable, row, mut clickable) in option_query.iter_mut() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }

        let Some((adapter, state)) = adapter_state_by_menu.get(&selectable.menu_entity).copied() else {
            clickable.region = Some(video_option_region(row.index));
            continue;
        };

        let visible = row_visible_in_viewport(
            &state,
            row.index,
            adapter.row_extent,
            adapter.leading_padding,
        );
        clickable.region = if visible {
            Some(video_option_region(row.index))
        } else {
            None
        };
    }
}

pub(super) fn ensure_video_top_row_visible(
    menu_entity: Entity,
    row: usize,
    root_query: &mut Query<
        (
            &ScrollableTableAdapter,
            &mut ScrollState,
            &mut ScrollFocusFollowLock,
        ),
        With<VideoTopOptionsScrollRoot>,
    >,
) {
    for (adapter, mut state, mut focus_lock) in root_query.iter_mut() {
        if adapter.menu_entity != menu_entity || row >= adapter.row_count {
            continue;
        }
        focus_lock.manual_override = false;
        focus_scroll_offset_to_row(&mut state, row, adapter.row_extent, adapter.leading_padding);
        break;
    }
}

fn row_top_and_bottom(
    row_index: usize,
    row_extent: f32,
    leading_padding: f32,
) -> (f32, f32) {
    let row_extent = row_extent.max(1.0);
    let leading_padding = leading_padding.max(0.0);
    let row_top = leading_padding + row_index as f32 * row_extent;
    let row_bottom = row_top + row_extent;
    (row_top, row_bottom)
}

fn row_visible_in_viewport(
    state: &ScrollState,
    row_index: usize,
    row_extent: f32,
    leading_padding: f32,
) -> bool {
    let (row_top, row_bottom) = row_top_and_bottom(row_index, row_extent, leading_padding);
    let viewport_top = state.offset_px;
    let viewport_bottom = state.offset_px + state.viewport_extent;
    row_bottom > viewport_top + VISIBILITY_EPSILON
        && row_top < viewport_bottom - VISIBILITY_EPSILON
}

fn focus_scroll_offset_to_row(
    state: &mut ScrollState,
    row_index: usize,
    row_extent: f32,
    leading_padding: f32,
) {
    let (row_top, row_bottom) = row_top_and_bottom(row_index, row_extent, leading_padding);
    let viewport_top = state.offset_px;
    let viewport_bottom = state.offset_px + state.viewport_extent;

    if row_top < viewport_top {
        state.offset_px = row_top;
    } else if row_bottom > viewport_bottom {
        state.offset_px = (row_bottom - state.viewport_extent).max(0.0);
    }

    state.max_offset = (state.content_extent - state.viewport_extent).max(0.0);
    state.offset_px = state.offset_px.clamp(0.0, state.max_offset);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::{focus_scroll_offset_to_row, row_visible_in_viewport};
    use super::*;
    use crate::systems::ui::scroll::ScrollState;

    #[test]
    fn focus_follow_scrolls_down_when_row_below_viewport() {
        let mut state = ScrollState {
            offset_px: 0.0,
            content_extent: 240.0,
            viewport_extent: 120.0,
            max_offset: 120.0,
        };
        focus_scroll_offset_to_row(&mut state, 3, 40.0, 0.0);
        assert_eq!(state.offset_px, 40.0);
    }

    #[test]
    fn focus_follow_scrolls_up_when_row_above_viewport() {
        let mut state = ScrollState {
            offset_px: 80.0,
            content_extent: 240.0,
            viewport_extent: 120.0,
            max_offset: 120.0,
        };
        focus_scroll_offset_to_row(&mut state, 1, 40.0, 0.0);
        assert_eq!(state.offset_px, 40.0);
    }

    #[test]
    fn focus_follow_honors_leading_padding() {
        let mut state = ScrollState {
            offset_px: 0.0,
            content_extent: 380.0,
            viewport_extent: 267.0,
            max_offset: 113.0,
        };
        focus_scroll_offset_to_row(&mut state, 7, 40.0, 60.0);
        assert!((state.offset_px - 113.0).abs() < 0.001);
    }

    #[test]
    fn row_visibility_respects_scroll_window() {
        let state = ScrollState {
            offset_px: 0.0,
            content_extent: 380.0,
            viewport_extent: 267.0,
            max_offset: 113.0,
        };
        assert!(!row_visible_in_viewport(&state, 7, 40.0, 60.0));
        assert!(row_visible_in_viewport(&state, 3, 40.0, 60.0));
    }

    #[test]
    fn non_navigation_key_does_not_clear_manual_override_lock() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<tabbed_menu::TabbedMenuFocusState>();
        app.add_systems(Update, sync_video_top_scroll_focus_follow);

        let menu_entity = app
            .world_mut()
            .spawn((
                MenuRoot {
                    host: MenuHost::Main,
                    gate: InteractionGate::PauseMenuOnly,
                },
                SelectableMenu::new(
                    0,
                    vec![KeyCode::ArrowUp],
                    vec![KeyCode::ArrowDown],
                    vec![KeyCode::Enter],
                    true,
                ),
            ))
            .id();
        let scroll_root = app
            .world_mut()
            .spawn((
                VideoTopOptionsScrollRoot,
                ScrollableTableAdapter::new(menu_entity, 8, 40.0, 60.0),
                ScrollState {
                    offset_px: 80.0,
                    content_extent: 500.0,
                    viewport_extent: 240.0,
                    max_offset: 260.0,
                },
                ScrollFocusFollowLock {
                    manual_override: true,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyA);
        app.update();

        let lock = app
            .world()
            .get::<ScrollFocusFollowLock>(scroll_root)
            .expect("focus lock");
        assert!(lock.manual_override);
    }
}
