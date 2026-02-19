use std::collections::HashMap;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::{
    data::states::PauseState,
    startup::cursor::CustomCursor,
    systems::{
        interaction::{
            interaction_gate_allows_for_owner, is_cursor_within_region, InteractionCapture,
            InteractionCaptureOwner, InteractionGate,
        },
        ui::layer::{self, UiLayer, UiLayerKind},
    },
};

use super::{
    geometry::{axis_extent, clamp_scroll_state, edge_auto_scroll_delta},
    ScrollAxis, ScrollFocusFollowLock, ScrollState, ScrollableContent,
    ScrollableContentBaseTranslation, ScrollableContentExtent, ScrollableItem, ScrollableRoot,
    ScrollableViewport, SCROLL_EPSILON, SCROLL_KEYBOARD_STEP_PX, SCROLL_PAGE_FACTOR,
    SCROLL_WHEEL_LINE_PX,
};

pub(super) fn sync_scroll_extents(
    mut root_query: Query<
        (
            Entity,
            &ScrollableRoot,
            &ScrollableViewport,
            &mut ScrollState,
            Option<&ScrollableContentExtent>,
        ),
        With<ScrollableRoot>,
    >,
    content_query: Query<(Entity, &ChildOf), With<ScrollableContent>>,
    item_query: Query<(&ScrollableItem, &ChildOf), Without<ScrollableRoot>>,
) {
    let mut content_by_root = HashMap::new();
    for (content_entity, parent) in content_query.iter() {
        content_by_root.insert(parent.parent(), content_entity);
    }

    let mut extent_by_content: HashMap<Entity, f32> = HashMap::new();
    for (item, parent) in item_query.iter() {
        let entry = extent_by_content.entry(parent.parent()).or_default();
        *entry += item.extent.max(0.0);
    }

    for (root_entity, root, viewport, mut state, explicit_extent) in root_query.iter_mut() {
        let viewport_extent = axis_extent(viewport.size, root.axis).max(0.0);
        let content_extent = if let Some(explicit) = explicit_extent {
            explicit.0.max(0.0)
        } else if let Some(content_entity) = content_by_root.get(&root_entity).copied() {
            extent_by_content
                .get(&content_entity)
                .copied()
                .unwrap_or(viewport_extent)
        } else {
            viewport_extent
        };

        state.viewport_extent = viewport_extent;
        state.content_extent = content_extent.max(viewport_extent);
        clamp_scroll_state(&mut state);
    }
}

pub(super) fn handle_scrollable_pointer_and_keyboard_input(
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    ui_layer_query: Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    cursor: Res<CustomCursor>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut root_query: Query<
        (
            Entity,
            &ScrollableRoot,
            &ScrollableViewport,
            &Transform,
            &GlobalTransform,
            &mut ScrollState,
            &mut ScrollFocusFollowLock,
            Option<&InteractionGate>,
            Option<&InheritedVisibility>,
        ),
        With<ScrollableRoot>,
    >,
) {
    let pause_state = pause_state.as_ref();
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state, &capture_query, &ui_layer_query);

    let mut wheel_vertical = 0.0;
    let mut wheel_horizontal = 0.0;
    for event in wheel_events.read() {
        let unit_scale = match event.unit {
            MouseScrollUnit::Line => SCROLL_WHEEL_LINE_PX,
            MouseScrollUnit::Pixel => 1.0,
        };
        wheel_vertical += event.y * unit_scale;
        wheel_horizontal += event.x * unit_scale;
    }

    let keyboard_navigation_requested = keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::PageUp)
        || keyboard_input.just_pressed(KeyCode::PageDown)
        || keyboard_input.just_pressed(KeyCode::Home)
        || keyboard_input.just_pressed(KeyCode::End);
    let cursor_position = cursor.position;

    let mut hovered_target: Option<(Entity, f32, u64)> = None;
    let mut edge_target: Option<(Entity, f32, u64, f32)> = None;
    let mut keyboard_target: Option<(Entity, f32, u64)> = None;
    for (entity, root, viewport, transform, global_transform, _, _, gate, inherited_visibility) in
        root_query.iter_mut()
    {
        if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
            continue;
        }
        if !interaction_gate_allows_for_owner(gate, pause_state, &capture_query, root.owner) {
            continue;
        }
        let active_layer_kind = active_layers
            .get(&root.owner)
            .map(|active_layer| active_layer.kind)
            .unwrap_or(UiLayerKind::Base);
        if active_layer_kind != root.input_layer {
            continue;
        }

        let keyboard_candidate = (entity, global_transform.translation().z, entity.to_bits());
        match keyboard_target {
            None => keyboard_target = Some(keyboard_candidate),
            Some((_, current_z, current_rank))
                if keyboard_candidate.1 > current_z
                    || (keyboard_candidate.1 == current_z
                        && keyboard_candidate.2 > current_rank) =>
            {
                keyboard_target = Some(keyboard_candidate);
            }
            _ => {}
        }

        let Some(cursor_position) = cursor_position else {
            continue;
        };
        let hovered = is_cursor_within_region(
            cursor_position,
            transform,
            global_transform,
            viewport.size,
            Vec2::ZERO,
        );
        if !hovered {
            if let Some(edge_delta) =
                edge_auto_scroll_delta(cursor_position, global_transform, viewport.size, root.axis)
            {
                let candidate = (
                    entity,
                    global_transform.translation().z,
                    entity.to_bits(),
                    edge_delta,
                );
                match edge_target {
                    None => edge_target = Some(candidate),
                    Some((_, current_z, current_rank, _))
                        if candidate.1 > current_z
                            || (candidate.1 == current_z && candidate.2 > current_rank) =>
                    {
                        edge_target = Some(candidate);
                    }
                    _ => {}
                }
            }
            continue;
        }

        let candidate = (entity, global_transform.translation().z, entity.to_bits());
        match hovered_target {
            None => hovered_target = Some(candidate),
            Some((_, current_z, current_rank))
                if candidate.1 > current_z
                    || (candidate.1 == current_z && candidate.2 > current_rank) =>
            {
                hovered_target = Some(candidate);
            }
            _ => {}
        }
    }

    let mut target_entity = None;
    let mut edge_delta = 0.0;
    let mut pointer_target_active = false;
    if let Some((entity, _, _, resolved_edge_delta)) =
        hovered_target.map(|(entity, z, rank)| (entity, z, rank, 0.0)).or(edge_target)
    {
        target_entity = Some(entity);
        edge_delta = resolved_edge_delta;
        pointer_target_active = true;
    } else if keyboard_navigation_requested {
        target_entity = keyboard_target.map(|(entity, _, _)| entity);
    }

    let Some(target_entity) = target_entity else {
        return;
    };

    let Ok((_, root, _, _, _, mut state, mut focus_lock, _, _)) = root_query.get_mut(target_entity)
    else {
        return;
    };
    if state.max_offset <= SCROLL_EPSILON {
        return;
    }

    let wheel_delta = match root.axis {
        ScrollAxis::Vertical => wheel_vertical,
        ScrollAxis::Horizontal => wheel_horizontal,
    };
    if pointer_target_active && wheel_delta.abs() > SCROLL_EPSILON {
        state.offset_px -= wheel_delta;
        focus_lock.manual_override = true;
    }
    if pointer_target_active && edge_delta.abs() > SCROLL_EPSILON {
        state.offset_px += edge_delta;
        focus_lock.manual_override = true;
    }

    let key_step = match root.axis {
        ScrollAxis::Vertical => {
            if keyboard_input.just_pressed(KeyCode::ArrowUp) {
                -SCROLL_KEYBOARD_STEP_PX
            } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
                SCROLL_KEYBOARD_STEP_PX
            } else if keyboard_input.just_pressed(KeyCode::PageUp) {
                -(state.viewport_extent * SCROLL_PAGE_FACTOR)
            } else if keyboard_input.just_pressed(KeyCode::PageDown) {
                state.viewport_extent * SCROLL_PAGE_FACTOR
            } else {
                0.0
            }
        }
        ScrollAxis::Horizontal => {
            if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
                -SCROLL_KEYBOARD_STEP_PX
            } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
                SCROLL_KEYBOARD_STEP_PX
            } else {
                0.0
            }
        }
    };

    if keyboard_input.just_pressed(KeyCode::Home) {
        state.offset_px = 0.0;
    } else if keyboard_input.just_pressed(KeyCode::End) {
        state.offset_px = state.max_offset;
    } else if key_step.abs() > SCROLL_EPSILON {
        state.offset_px += key_step;
    }

    clamp_scroll_state(&mut state);
}

pub(super) fn sync_scroll_content_offsets(
    mut commands: Commands,
    root_query: Query<(Entity, &ScrollableRoot, &ScrollState), With<ScrollableRoot>>,
    mut content_query: Query<
        (
            Entity,
            &ChildOf,
            &mut Transform,
            Option<&ScrollableContentBaseTranslation>,
        ),
        With<ScrollableContent>,
    >,
) {
    let root_state: HashMap<Entity, (ScrollAxis, f32)> = root_query
        .iter()
        .map(|(entity, root, state)| (entity, (root.axis, state.offset_px)))
        .collect();

    for (content_entity, parent, mut transform, base_translation) in content_query.iter_mut() {
        let Some((axis, offset)) = root_state.get(&parent.parent()).copied() else {
            continue;
        };
        let base = if let Some(base) = base_translation {
            base.0
        } else {
            let base = transform.translation;
            commands
                .entity(content_entity)
                .insert(ScrollableContentBaseTranslation(base));
            base
        };

        match axis {
            ScrollAxis::Vertical => {
                transform.translation.x = base.x;
                // Positive vertical scroll offset should reveal lower rows,
                // which means moving content upward in world space (+Y in Bevy).
                transform.translation.y = base.y + offset;
            }
            ScrollAxis::Horizontal => {
                transform.translation.x = base.x - offset;
                transform.translation.y = base.y;
            }
        }
        transform.translation.z = base.z;
    }
}
