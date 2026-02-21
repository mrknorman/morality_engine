use std::collections::HashMap;

use bevy::{
    input::gestures::PinchGesture,
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::{
    data::states::PauseState,
    startup::cursor::CustomCursor,
    systems::{
        interaction::{
            focused_scope_owner_from_registry, interaction_gate_allows_for_owner,
            is_cursor_within_region, resolve_focused_scope_owner, scoped_owner_has_focus,
            InteractionCapture, InteractionCaptureOwner, InteractionGate, SelectableScopeOwner,
        },
        ui::layer::{self, UiLayer, UiLayerKind},
    },
};

use super::{
    geometry::{axis_extent, clamp_scroll_state, edge_auto_scroll_delta},
    ScrollAxis, ScrollBar, ScrollFocusFollowLock, ScrollState, ScrollZoomConfig, ScrollZoomState,
    ScrollableContent, ScrollableContentBaseScale, ScrollableContentBaseTranslation,
    ScrollableContentExtent, ScrollableItem, ScrollableRoot, ScrollableViewport, SCROLL_EPSILON,
    SCROLL_KEYBOARD_STEP_PX, SCROLL_PAGE_FACTOR, SCROLL_WHEEL_LINE_PX, SCROLL_ZOOM_EPSILON,
};

pub(super) fn sync_scroll_extents(
    mut root_query: Query<
        (
            Entity,
            &ScrollableRoot,
            &ScrollableViewport,
            &mut ScrollState,
            &ScrollZoomState,
            Option<&ScrollableContentExtent>,
        ),
        With<ScrollableRoot>,
    >,
    content_query: Query<(Entity, &ChildOf), With<ScrollableContent>>,
    item_query: Query<(&ScrollableItem, &ChildOf), Without<ScrollableRoot>>,
) {
    // Query contract:
    // - root state is mutated only through `root_query`.
    // - content/item queries are read-only and disjoint by role
    //   (`ScrollableContent` vs `ScrollableItem`), preventing aliasing.
    let mut content_by_root = HashMap::new();
    for (content_entity, parent) in content_query.iter() {
        content_by_root.insert(parent.parent(), content_entity);
    }

    let mut extent_by_content: HashMap<Entity, f32> = HashMap::new();
    for (item, parent) in item_query.iter() {
        let entry = extent_by_content.entry(parent.parent()).or_default();
        *entry += item.extent.max(0.0);
    }

    for (root_entity, root, viewport, mut state, zoom, explicit_extent) in root_query.iter_mut() {
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
        state.content_extent =
            (content_extent * zoom.scale.max(SCROLL_ZOOM_EPSILON)).max(viewport_extent);
        clamp_scroll_state(&mut state);
    }
}

pub(super) fn handle_scrollable_pointer_and_keyboard_input(
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    ui_layer_query: Query<(
        Entity,
        &UiLayer,
        Option<&Visibility>,
        Option<&InteractionGate>,
    )>,
    scope_owner_query: Query<(Option<&InteractionGate>, &SelectableScopeOwner)>,
    owner_transform_query: Query<(&GlobalTransform, Option<&InheritedVisibility>), Without<TextSpan>>,
    cursor: Res<CustomCursor>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut pinch_events: MessageReader<PinchGesture>,
    root_parent_query: Query<&ChildOf, With<ScrollableRoot>>,
    scrollbar_query: Query<&ScrollBar>,
    mut root_query: Query<
        (
            Entity,
            &ScrollableRoot,
            &ScrollableViewport,
            &Transform,
            &GlobalTransform,
            &mut ScrollState,
            &mut ScrollFocusFollowLock,
            &mut ScrollZoomState,
            &ScrollZoomConfig,
            Option<&InteractionGate>,
            Option<&InheritedVisibility>,
        ),
        With<ScrollableRoot>,
    >,
) {
    // Query contract:
    // - active-layer arbitration queries are read-only
    //   (`capture_query`, `ui_layer_query`).
    // - scroll state/focus lock mutations are isolated to `root_query`.
    // This keeps pointer+keyboard reduction deterministic and query-safe.
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
    let mut pinch_delta = 0.0;
    for event in pinch_events.read() {
        pinch_delta += event.0;
    }

    let keyboard_navigation_requested = keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::ArrowRight)
        || keyboard_input.just_pressed(KeyCode::PageUp)
        || keyboard_input.just_pressed(KeyCode::PageDown)
        || keyboard_input.just_pressed(KeyCode::Home)
        || keyboard_input.just_pressed(KeyCode::End);
    let vertical_navigation_requested = keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::ArrowDown)
        || keyboard_input.just_pressed(KeyCode::PageUp)
        || keyboard_input.just_pressed(KeyCode::PageDown)
        || keyboard_input.just_pressed(KeyCode::Home)
        || keyboard_input.just_pressed(KeyCode::End);
    let horizontal_navigation_requested = keyboard_input.just_pressed(KeyCode::ArrowLeft)
        || keyboard_input.just_pressed(KeyCode::ArrowRight);
    let control_held = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);
    let zoom_in_requested = control_held
        && (keyboard_input.just_pressed(KeyCode::Equal)
            || keyboard_input.just_pressed(KeyCode::NumpadAdd)
            || keyboard_input.just_pressed(KeyCode::NumpadEqual));
    let zoom_out_requested = control_held
        && (keyboard_input.just_pressed(KeyCode::Minus)
            || keyboard_input.just_pressed(KeyCode::NumpadSubtract));
    let keyboard_zoom_requested = zoom_in_requested || zoom_out_requested;
    let pinch_zoom_requested = pinch_delta.abs() > SCROLL_EPSILON;
    let cursor_position = cursor.position;

    fn scroll_root_depth(entity: Entity, root_parent_query: &Query<&ChildOf, With<ScrollableRoot>>) -> u32 {
        let mut depth = 0u32;
        let mut cursor = entity;
        while let Ok(parent) = root_parent_query.get(cursor) {
            depth += 1;
            cursor = parent.parent();
        }
        depth
    }

    #[derive(Clone, Copy)]
    struct Candidate {
        entity: Entity,
        owner: Entity,
        z: f32,
        rank: u64,
        depth: u32,
        viewport_extent: f32,
    }

    #[derive(Clone, Copy)]
    struct EdgeCandidate {
        candidate: Candidate,
        edge_delta: f32,
    }

    fn replace_candidate(current: Option<Candidate>, next: Candidate) -> Option<Candidate> {
        match current {
            None => Some(next),
            Some(current)
                if next.z > current.z || (next.z == current.z && next.rank > current.rank) =>
            {
                Some(next)
            }
            Some(current) => Some(current),
        }
    }

    fn replace_keyboard_candidate(current: Option<Candidate>, next: Candidate) -> Option<Candidate> {
        match current {
            None => Some(next),
            Some(current) => {
                let replace = (next.depth < current.depth)
                    || (next.depth == current.depth && next.viewport_extent > current.viewport_extent)
                    || (next.depth == current.depth
                        && (next.viewport_extent - current.viewport_extent).abs() <= f32::EPSILON
                        && (next.z > current.z
                            || (next.z == current.z && next.rank > current.rank)));
                if replace {
                    Some(next)
                } else {
                    Some(current)
                }
            }
        }
    }

    fn replace_edge_candidate(
        current: Option<EdgeCandidate>,
        next: EdgeCandidate,
    ) -> Option<EdgeCandidate> {
        match current {
            None => Some(next),
            Some(current)
                if next.candidate.z > current.candidate.z
                    || (next.candidate.z == current.candidate.z
                        && next.candidate.rank > current.candidate.rank) =>
            {
                Some(next)
            }
            Some(current) => Some(current),
        }
    }

    let mut scoped_owner_candidates: Vec<(Entity, f32)> = Vec::new();
    let mut hovered_vertical: Option<Candidate> = None;
    let mut edge_vertical: Option<EdgeCandidate> = None;
    let mut keyboard_vertical: Option<Candidate> = None;
    let mut hovered_horizontal: Option<Candidate> = None;
    let mut edge_horizontal: Option<EdgeCandidate> = None;
    let mut keyboard_horizontal: Option<Candidate> = None;
    let roots_with_scrollbars: std::collections::HashSet<Entity> =
        scrollbar_query.iter().map(|scrollbar| scrollbar.scrollable_root).collect();
    for (
        entity,
        root,
        viewport,
        transform,
        global_transform,
        _,
        _,
        _,
        _,
        gate,
        inherited_visibility,
    ) in
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

        let z = global_transform.translation().z;
        scoped_owner_candidates.push((root.owner, z));

        let keyboard_candidate = Candidate {
            entity,
            owner: root.owner,
            z,
            rank: entity.to_bits(),
            depth: scroll_root_depth(entity, &root_parent_query),
            viewport_extent: axis_extent(viewport.size, root.axis),
        };
        match root.axis {
            ScrollAxis::Vertical => {
                keyboard_vertical = replace_keyboard_candidate(keyboard_vertical, keyboard_candidate)
            }
            ScrollAxis::Horizontal => {
                keyboard_horizontal =
                    replace_keyboard_candidate(keyboard_horizontal, keyboard_candidate)
            }
        };

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
            if roots_with_scrollbars.contains(&entity) {
                continue;
            }
            if let Some(edge_delta) = edge_auto_scroll_delta(
                cursor_position,
                global_transform,
                viewport.size,
                root.axis,
                root.edge_zone_inside_px,
                root.edge_zone_outside_px,
            ) {
                let candidate = EdgeCandidate {
                    candidate: Candidate {
                        entity,
                        owner: root.owner,
                        z,
                        rank: entity.to_bits(),
                        depth: scroll_root_depth(entity, &root_parent_query),
                        viewport_extent: axis_extent(viewport.size, root.axis),
                    },
                    edge_delta,
                };
                match root.axis {
                    ScrollAxis::Vertical => edge_vertical = replace_edge_candidate(edge_vertical, candidate),
                    ScrollAxis::Horizontal => {
                        edge_horizontal = replace_edge_candidate(edge_horizontal, candidate)
                    }
                };
            }
            continue;
        }

        let candidate = Candidate {
            entity,
            owner: root.owner,
            z,
            rank: entity.to_bits(),
            depth: scroll_root_depth(entity, &root_parent_query),
            viewport_extent: axis_extent(viewport.size, root.axis),
        };
        match root.axis {
            ScrollAxis::Vertical => hovered_vertical = replace_candidate(hovered_vertical, candidate),
            ScrollAxis::Horizontal => {
                hovered_horizontal = replace_candidate(hovered_horizontal, candidate)
            }
        };
    }

    let focused_scoped_owner =
        focused_scope_owner_from_registry(pause_state, &capture_query, &scope_owner_query, &owner_transform_query)
            .or_else(|| resolve_focused_scope_owner(scoped_owner_candidates));

    let mut vertical_target: Option<(Entity, f32, bool)> = None;
    if let Some(candidate) = hovered_vertical {
        if scoped_owner_has_focus(Some(candidate.owner), focused_scoped_owner) {
            vertical_target = Some((candidate.entity, 0.0, true));
        }
    } else if let Some(edge_candidate) = edge_vertical {
        if scoped_owner_has_focus(Some(edge_candidate.candidate.owner), focused_scoped_owner) {
            vertical_target = Some((edge_candidate.candidate.entity, edge_candidate.edge_delta, true));
        }
    } else if keyboard_navigation_requested && vertical_navigation_requested {
        if let Some(candidate) = keyboard_vertical {
            if scoped_owner_has_focus(Some(candidate.owner), focused_scoped_owner) {
                vertical_target = Some((candidate.entity, 0.0, false));
            }
        }
    }

    let mut horizontal_target: Option<(Entity, f32, bool)> = None;
    if let Some(candidate) = hovered_horizontal {
        if scoped_owner_has_focus(Some(candidate.owner), focused_scoped_owner) {
            horizontal_target = Some((candidate.entity, 0.0, true));
        }
    } else if let Some(edge_candidate) = edge_horizontal {
        if scoped_owner_has_focus(Some(edge_candidate.candidate.owner), focused_scoped_owner) {
            horizontal_target = Some((edge_candidate.candidate.entity, edge_candidate.edge_delta, true));
        }
    } else if keyboard_navigation_requested && horizontal_navigation_requested {
        if let Some(candidate) = keyboard_horizontal {
            if scoped_owner_has_focus(Some(candidate.owner), focused_scoped_owner) {
                horizontal_target = Some((candidate.entity, 0.0, false));
            }
        }
    }

    if let Some((target_entity, edge_delta, pointer_target_active)) = vertical_target {
        if let Ok((_, root, _, _, _, mut state, mut focus_lock, _, _, _, _)) =
            root_query.get_mut(target_entity)
        {
            if root.axis == ScrollAxis::Vertical && state.max_offset > SCROLL_EPSILON {
                if pointer_target_active && wheel_vertical.abs() > SCROLL_EPSILON {
                    state.offset_px -= wheel_vertical;
                    focus_lock.manual_override = true;
                }
                if pointer_target_active && edge_delta.abs() > SCROLL_EPSILON {
                    state.offset_px += edge_delta;
                    focus_lock.manual_override = true;
                }

                let key_step = if keyboard_input.just_pressed(KeyCode::ArrowUp) {
                    -SCROLL_KEYBOARD_STEP_PX
                } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
                    SCROLL_KEYBOARD_STEP_PX
                } else if keyboard_input.just_pressed(KeyCode::PageUp) {
                    -(state.viewport_extent * SCROLL_PAGE_FACTOR)
                } else if keyboard_input.just_pressed(KeyCode::PageDown) {
                    state.viewport_extent * SCROLL_PAGE_FACTOR
                } else {
                    0.0
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
        }
    }

    if let Some((target_entity, edge_delta, pointer_target_active)) = horizontal_target {
        if let Ok((_, root, _, _, _, mut state, mut focus_lock, _, _, _, _)) =
            root_query.get_mut(target_entity)
        {
            if root.axis == ScrollAxis::Horizontal && state.max_offset > SCROLL_EPSILON {
                if pointer_target_active && wheel_horizontal.abs() > SCROLL_EPSILON {
                    state.offset_px -= wheel_horizontal;
                    focus_lock.manual_override = true;
                }
                if pointer_target_active && edge_delta.abs() > SCROLL_EPSILON {
                    state.offset_px += edge_delta;
                    focus_lock.manual_override = true;
                }

                let key_step = if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
                    -SCROLL_KEYBOARD_STEP_PX
                } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
                    SCROLL_KEYBOARD_STEP_PX
                } else {
                    0.0
                };
                if key_step.abs() > SCROLL_EPSILON {
                    state.offset_px += key_step;
                }

                clamp_scroll_state(&mut state);
            }
        }
    }

    if keyboard_zoom_requested || pinch_zoom_requested {
        let zoom_target = if let Some(candidate) = hovered_vertical {
            if scoped_owner_has_focus(Some(candidate.owner), focused_scoped_owner) {
                Some(candidate.entity)
            } else {
                None
            }
        } else if let Some(candidate) = keyboard_vertical {
            if scoped_owner_has_focus(Some(candidate.owner), focused_scoped_owner) {
                Some(candidate.entity)
            } else {
                None
            }
        } else {
            None
        };

        let mut owner_zoom_update: Option<(Entity, f32)> = None;
        if let Some(target_entity) = zoom_target {
            if let Ok((_, root, _, _, _, _, mut focus_lock, mut zoom_state, zoom_config, _, _)) =
                root_query.get_mut(target_entity)
            {
                if root.axis == ScrollAxis::Vertical && zoom_config.enabled {
                    let mut next_scale = zoom_state.scale;
                    if zoom_in_requested {
                        next_scale += zoom_config.keyboard_step;
                    }
                    if zoom_out_requested {
                        next_scale -= zoom_config.keyboard_step;
                    }
                    if pinch_zoom_requested {
                        next_scale *=
                            (1.0 + pinch_delta * zoom_config.pinch_sensitivity).max(0.1);
                    }

                    let min_scale = zoom_config.min_scale.min(zoom_config.max_scale);
                    let max_scale = zoom_config.max_scale.max(min_scale + SCROLL_ZOOM_EPSILON);
                    let clamped_scale = next_scale.clamp(min_scale, max_scale);
                    if (clamped_scale - zoom_state.scale).abs() > SCROLL_ZOOM_EPSILON {
                        zoom_state.scale = clamped_scale;
                        focus_lock.manual_override = true;
                        owner_zoom_update = Some((root.owner, clamped_scale));
                    }
                }
            }
        }

        if let Some((owner, clamped_scale)) = owner_zoom_update {
            for (_, root, _, _, _, _, mut focus_lock, mut zoom_state, zoom_config, _, _) in
                root_query.iter_mut()
            {
                if !zoom_config.enabled {
                    continue;
                }
                if root.owner != owner {
                    continue;
                }
                if (zoom_state.scale - clamped_scale).abs() <= SCROLL_ZOOM_EPSILON {
                    continue;
                }
                zoom_state.scale = clamped_scale;
                focus_lock.manual_override = true;
            }
        }
    }
}

pub(super) fn sync_scroll_content_offsets(
    mut commands: Commands,
    root_query: Query<(Entity, &ScrollableRoot, &ScrollState, &ScrollZoomState), With<ScrollableRoot>>,
    mut content_query: Query<
        (
            Entity,
            &ChildOf,
            &mut Transform,
            Option<&ScrollableContentBaseTranslation>,
            Option<&ScrollableContentBaseScale>,
        ),
        With<ScrollableContent>,
    >,
) {
    // Query contract:
    // - root scroll state is read-only (`root_query`).
    // - content transform mutations are isolated to entities marked
    //   `ScrollableContent`.
    let root_state: HashMap<Entity, (ScrollAxis, f32, f32)> = root_query
        .iter()
        .map(|(entity, root, state, zoom)| (entity, (root.axis, state.offset_px, zoom.scale)))
        .collect();

    for (content_entity, parent, mut transform, base_translation, base_scale) in
        content_query.iter_mut()
    {
        let Some((axis, offset, zoom_scale)) = root_state.get(&parent.parent()).copied() else {
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
        let base_scale = if let Some(base_scale) = base_scale {
            base_scale.0
        } else {
            let base_scale = transform.scale;
            commands
                .entity(content_entity)
                .insert(ScrollableContentBaseScale(base_scale));
            base_scale
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
        transform.scale = Vec3::new(
            base_scale.x * zoom_scale,
            base_scale.y * zoom_scale,
            base_scale.z,
        );
    }
}
