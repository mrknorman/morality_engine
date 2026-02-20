use bevy::prelude::*;

use crate::{
    data::states::PauseState,
    entities::sprites::compound::{BorderedRectangle, HollowRectangle},
    startup::cursor::CustomCursor,
    systems::{
        interaction::{
            interaction_gate_allows_for_owner, Clickable, InteractionCapture,
            InteractionCaptureOwner, InteractionGate, ScrollUiActions,
        },
        ui::layer::{self, UiLayer, UiLayerKind},
    },
};

use super::{
    geometry::{axis_extent, axis_value_vec2, axis_value_vec3, clamp_scroll_state},
    scrollbar_math::{
        offset_from_thumb_center, scrollbar_click_region, thumb_center_for_offset,
        thumb_extent_for_state,
    },
    ScrollAxis, ScrollBar, ScrollBarDragState, ScrollBarParts, ScrollBarThumb, ScrollBarTrack,
    ScrollFocusFollowLock, ScrollState, ScrollableRoot, ScrollableViewport, SCROLL_EPSILON,
    SCROLL_PAGE_FACTOR, SCROLLBAR_BORDER_THICKNESS, SCROLLBAR_INTERACTION_Z,
    SCROLLBAR_THUMB_INSET, SCROLLBAR_THUMB_Z, SCROLLBAR_TRACK_FILL_COLOR, SCROLLBAR_TRACK_Z,
};

pub(super) fn ensure_scrollbar_parts(
    mut commands: Commands,
    root_gate_query: Query<Option<&InteractionGate>, With<ScrollableRoot>>,
    scrollbar_query: Query<(Entity, &ScrollBar, Option<&ChildOf>), Without<ScrollBarParts>>,
) {
    for (scrollbar_entity, scrollbar, parent) in scrollbar_query.iter() {
        if parent.is_none_or(|parent| parent.parent() != scrollbar.scrollable_root) {
            commands
                .entity(scrollbar.scrollable_root)
                .add_child(scrollbar_entity);
        }

        let gate = root_gate_query
            .get(scrollbar.scrollable_root)
            .ok()
            .flatten()
            .copied()
            .unwrap_or_default();

        let mut track_entity = None;
        let mut thumb_entity = None;
        commands.entity(scrollbar_entity).with_children(|parent| {
            let track = parent
                .spawn((
                    Name::new("scrollbar_track"),
                    ScrollBarTrack,
                    gate,
                    Clickable::with_region(vec![ScrollUiActions::Activate], Vec2::splat(10.0)),
                    BorderedRectangle {
                        boundary: HollowRectangle {
                            dimensions: Vec2::new(scrollbar.width, 64.0),
                            thickness: SCROLLBAR_BORDER_THICKNESS,
                            color: scrollbar.track_color,
                            ..default()
                        },
                        fill_color: SCROLLBAR_TRACK_FILL_COLOR,
                    },
                    Transform::from_xyz(0.0, 0.0, SCROLLBAR_TRACK_Z),
                ))
                .id();
            track_entity = Some(track);

            let thumb = parent
                .spawn((
                    Name::new("scrollbar_thumb"),
                    ScrollBarThumb,
                    gate,
                    Clickable::with_region(vec![ScrollUiActions::Activate], Vec2::splat(10.0)),
                    BorderedRectangle {
                        boundary: HollowRectangle {
                            dimensions: Vec2::new(scrollbar.width, 24.0),
                            thickness: SCROLLBAR_BORDER_THICKNESS,
                            color: scrollbar.track_color,
                            ..default()
                        },
                        fill_color: scrollbar.thumb_color,
                    },
                    Transform::from_xyz(0.0, 0.0, SCROLLBAR_THUMB_Z),
                ))
                .id();
            thumb_entity = Some(thumb);
        });

        let Some(track_entity) = track_entity else {
            continue;
        };
        let Some(thumb_entity) = thumb_entity else {
            continue;
        };
        commands.entity(scrollbar_entity).insert((
            ScrollBarParts {
                track: track_entity,
                thumb: thumb_entity,
            },
        ));
    }
}

pub(super) fn sync_scrollbar_visuals(
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    root_query: Query<
        (&ScrollableRoot, &ScrollableViewport, &ScrollState, Option<&InheritedVisibility>),
        With<ScrollableRoot>,
    >,
    mut scrollbar_query: Query<
        (
            &ScrollBar,
            &ScrollBarParts,
            &mut Transform,
            Option<&InheritedVisibility>,
        ),
        (Without<ScrollBarTrack>, Without<ScrollBarThumb>),
    >,
    mut query_set: ParamSet<(
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
        Query<
            (
                &mut BorderedRectangle,
                &mut Transform,
                &mut Visibility,
                &mut Clickable<ScrollUiActions>,
            ),
            (With<ScrollBarTrack>, Without<ScrollBarThumb>),
        >,
        Query<
            (
                &mut BorderedRectangle,
                &mut Transform,
                &mut Visibility,
                &mut Clickable<ScrollUiActions>,
            ),
            (With<ScrollBarThumb>, Without<ScrollBarTrack>),
        >,
    )>,
) {
    // Query contract:
    // - Ui-layer visibility reads (p0) are separated from track/thumb visibility
    //   mutations (p1/p2) via ParamSet to avoid B0001 aliasing.
    // - Scrollbar root transforms are mutated via `scrollbar_query`.
    // - Track/thumb transforms and clickables are mutated via disjoint
    //   `Without`-guarded queries (p1/p2).
    // This prevents Bevy B0001 aliasing across shared components.
    let active_layers = {
        let ui_layer_query = query_set.p0();
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &ui_layer_query)
    };
    for (scrollbar, parts, mut scrollbar_transform, scrollbar_visibility) in scrollbar_query.iter_mut()
    {
        let Ok((root, viewport, state, root_visibility)) = root_query.get(scrollbar.scrollable_root)
        else {
            continue;
        };
        let root_visible = root_visibility.is_none_or(|visibility| visibility.get());
        let scrollbar_visible = scrollbar_visibility.is_none_or(|visibility| visibility.get());
        let visible = root_visible && scrollbar_visible;

        let track_extent = axis_extent(viewport.size, root.axis).max(1.0);
        let thickness = scrollbar.width.max(2.0);
        let thumb_extent = thumb_extent_for_state(
            track_extent,
            state.viewport_extent,
            state.content_extent,
            scrollbar.min_thumb_extent,
        );
        let thumb_center = thumb_center_for_offset(
            track_extent,
            thumb_extent,
            state.offset_px,
            state.max_offset,
            root.axis,
        );

        let active_layer_matches_input = active_layers
            .get(&root.owner)
            .map(|active_layer| active_layer.kind)
            .unwrap_or(UiLayerKind::Base)
            == root.input_layer;
        let has_scroll_range =
            state.max_offset > SCROLL_EPSILON && state.content_extent > state.viewport_extent;
        let part_visibility = if visible && has_scroll_range {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let effective_part_visibility = if active_layer_matches_input {
            part_visibility
        } else {
            Visibility::Hidden
        };

        let track_outer_size;
        let thumb_outer_size;
        let track_translation;
        let thumb_translation;
        match root.axis {
            ScrollAxis::Vertical => {
                scrollbar_transform.translation.x =
                    viewport.size.x * 0.5 + scrollbar.margin + thickness * 0.5;
                scrollbar_transform.translation.y = 0.0;
                scrollbar_transform.translation.z = SCROLLBAR_INTERACTION_Z;
                track_outer_size = Vec2::new(thickness, track_extent);
                track_translation = Vec3::new(0.0, 0.0, SCROLLBAR_TRACK_Z);
                thumb_outer_size = Vec2::new(
                    (thickness - SCROLLBAR_THUMB_INSET * 2.0).max(1.0),
                    thumb_extent,
                );
                thumb_translation = Vec3::new(0.0, thumb_center, SCROLLBAR_THUMB_Z);
            }
            ScrollAxis::Horizontal => {
                scrollbar_transform.translation.x = 0.0;
                scrollbar_transform.translation.y =
                    -viewport.size.y * 0.5 - scrollbar.margin - thickness * 0.5;
                scrollbar_transform.translation.z = SCROLLBAR_INTERACTION_Z;
                track_outer_size = Vec2::new(track_extent, thickness);
                track_translation = Vec3::new(0.0, 0.0, SCROLLBAR_TRACK_Z);
                thumb_outer_size = Vec2::new(
                    thumb_extent,
                    (thickness - SCROLLBAR_THUMB_INSET * 2.0).max(1.0),
                );
                thumb_translation = Vec3::new(thumb_center, 0.0, SCROLLBAR_THUMB_Z);
            }
        }

        {
            let mut track_query = query_set.p1();
            let Ok((mut track_rect, mut track_transform, mut track_visibility, mut track_clickable)) =
                track_query.get_mut(parts.track)
            else {
                continue;
            };
            *track_visibility = effective_part_visibility;
            track_transform.translation = track_translation;
            track_rect.boundary.dimensions = track_outer_size;
            track_rect.boundary.thickness = SCROLLBAR_BORDER_THICKNESS;
            track_rect.boundary.color = scrollbar.track_color;
            track_rect.fill_color = SCROLLBAR_TRACK_FILL_COLOR;
            track_clickable.region = Some(scrollbar_click_region(track_outer_size, root.axis));
        }

        {
            let mut thumb_query = query_set.p2();
            let Ok((mut thumb_rect, mut thumb_transform, mut thumb_visibility, mut thumb_clickable)) =
                thumb_query.get_mut(parts.thumb)
            else {
                continue;
            };
            *thumb_visibility = effective_part_visibility;
            thumb_transform.translation = thumb_translation;
            thumb_rect.boundary.dimensions = thumb_outer_size;
            thumb_rect.boundary.thickness = SCROLLBAR_BORDER_THICKNESS;
            thumb_rect.boundary.color = scrollbar.track_color;
            thumb_rect.fill_color = scrollbar.thumb_color;
            thumb_clickable.region = Some(scrollbar_click_region(thumb_outer_size, root.axis));
        }
    }
}

pub(super) fn handle_scrollbar_input(
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    ui_layer_query: Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
    cursor: Res<CustomCursor>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    root_meta_query: Query<
        (
            &ScrollableRoot,
            &ScrollableViewport,
            Option<&InteractionGate>,
            Option<&InheritedVisibility>,
        ),
        With<ScrollableRoot>,
    >,
    mut root_state_queries: ParamSet<(
        Query<&ScrollState, With<ScrollableRoot>>,
        Query<&mut ScrollState, With<ScrollableRoot>>,
    )>,
    mut focus_lock_query: Query<&mut ScrollFocusFollowLock, With<ScrollableRoot>>,
    mut scrollbar_query: Query<
        (
            Entity,
            &ScrollBar,
            &ScrollBarParts,
            &mut ScrollBarDragState,
            Option<&InheritedVisibility>,
        ),
    >,
    mut track_query: Query<
        (
            &mut Clickable<ScrollUiActions>,
            &GlobalTransform,
            &Transform,
            &BorderedRectangle,
            Option<&InheritedVisibility>,
        ),
        (With<ScrollBarTrack>, Without<ScrollBarThumb>),
    >,
    mut thumb_query: Query<
        (
            &mut Clickable<ScrollUiActions>,
            &GlobalTransform,
            &Transform,
            &BorderedRectangle,
            Option<&InheritedVisibility>,
        ),
        (With<ScrollBarThumb>, Without<ScrollBarTrack>),
    >,
) {
    // Query contract:
    // - Track and thumb mutation queries are disjoint through complementary
    //   `Without` filters.
    // - Scroll state reads/writes are isolated through `ParamSet`.
    // This keeps drag/track paging paths query-safe under rapid interaction.
    let pause_state = pause_state.as_ref();
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state, &capture_query, &ui_layer_query);
    let cursor_position = cursor.position;
    let mouse_down = mouse_input.pressed(MouseButton::Left);

    for (_, _, _, mut drag_state, _) in scrollbar_query.iter_mut() {
        if !mouse_down {
            drag_state.dragging = false;
        }
    }

    for (_scrollbar_entity, scrollbar, parts, mut drag_state, scrollbar_visibility) in
        scrollbar_query.iter_mut()
    {
        let Ok((root, _viewport, gate, root_visibility)) = root_meta_query.get(scrollbar.scrollable_root)
        else {
            continue;
        };
        if root_visibility.is_some_and(|visibility| !visibility.get())
            || scrollbar_visibility.is_some_and(|visibility| !visibility.get())
        {
            drag_state.dragging = false;
            continue;
        }
        if !interaction_gate_allows_for_owner(gate, pause_state, &capture_query, root.owner) {
            drag_state.dragging = false;
            continue;
        }
        let active_layer_kind = active_layers
            .get(&root.owner)
            .map(|active_layer| active_layer.kind)
            .unwrap_or(UiLayerKind::Base);
        if active_layer_kind != root.input_layer {
            drag_state.dragging = false;
            continue;
        }

        let state_snapshot = {
            let state_query = root_state_queries.p0();
            let Ok(state) = state_query.get(scrollbar.scrollable_root) else {
                drag_state.dragging = false;
                continue;
            };
            *state
        };

        if state_snapshot.max_offset <= SCROLL_EPSILON {
            drag_state.dragging = false;
        }

        let Ok((mut track_clickable, track_global, _track_transform, track_sprite, track_visibility)) =
            track_query.get_mut(parts.track)
        else {
            continue;
        };
        let Ok((mut thumb_clickable, thumb_global, _thumb_transform, thumb_sprite, thumb_visibility)) =
            thumb_query.get_mut(parts.thumb)
        else {
            continue;
        };
        if track_visibility.is_some_and(|visibility| !visibility.get())
            || thumb_visibility.is_some_and(|visibility| !visibility.get())
        {
            drag_state.dragging = false;
            track_clickable.triggered = false;
            thumb_clickable.triggered = false;
            continue;
        }

        let track_size = track_sprite.boundary.dimensions.max(Vec2::ZERO);
        let thumb_size = thumb_sprite.boundary.dimensions.max(Vec2::ZERO);
        let track_extent = axis_extent(track_size, root.axis).max(1.0);
        let thumb_extent = axis_extent(thumb_size, root.axis).max(1.0);

        let thumb_clicked = thumb_clickable.triggered;
        let track_clicked = track_clickable.triggered;
        thumb_clickable.triggered = false;
        track_clickable.triggered = false;

        if thumb_clicked {
            if let Some(cursor_position) = cursor_position {
                let thumb_center_world = axis_value_vec3(thumb_global.translation(), root.axis);
                drag_state.dragging = true;
                drag_state.grab_offset =
                    axis_value_vec2(cursor_position, root.axis) - thumb_center_world;
                if let Ok(mut focus_lock) = focus_lock_query.get_mut(scrollbar.scrollable_root) {
                    focus_lock.manual_override = true;
                }
            }
        } else if track_clicked {
            if let Some(cursor_position) = cursor_position {
                let mut state_query = root_state_queries.p1();
                let Ok(mut state) = state_query.get_mut(scrollbar.scrollable_root) else {
                    continue;
                };
                let thumb_center_world = axis_value_vec3(thumb_global.translation(), root.axis);
                let cursor_axis = axis_value_vec2(cursor_position, root.axis);
                let page = state.viewport_extent * SCROLL_PAGE_FACTOR;
                match root.axis {
                    ScrollAxis::Vertical => {
                        if cursor_axis < thumb_center_world {
                            state.offset_px += page;
                        } else {
                            state.offset_px -= page;
                        }
                    }
                    ScrollAxis::Horizontal => {
                        if cursor_axis > thumb_center_world {
                            state.offset_px += page;
                        } else {
                            state.offset_px -= page;
                        }
                    }
                }
                clamp_scroll_state(&mut state);
                if let Ok(mut focus_lock) = focus_lock_query.get_mut(scrollbar.scrollable_root) {
                    focus_lock.manual_override = true;
                }
            }
        }

        if !drag_state.dragging || !mouse_down {
            continue;
        }

        let Some(cursor_position) = cursor_position else {
            continue;
        };
        let cursor_axis = axis_value_vec2(cursor_position, root.axis);
        let track_center_world = axis_value_vec3(track_global.translation(), root.axis);
        let target_world_center = cursor_axis - drag_state.grab_offset;
        let target_local_center = target_world_center - track_center_world;

        let mut state_query = root_state_queries.p1();
        let Ok(mut state) = state_query.get_mut(scrollbar.scrollable_root) else {
            continue;
        };
        state.offset_px = offset_from_thumb_center(
            track_extent,
            thumb_extent,
            target_local_center,
            state.max_offset,
            root.axis,
        );
        clamp_scroll_state(&mut state);
        if let Ok(mut focus_lock) = focus_lock_query.get_mut(scrollbar.scrollable_root) {
            focus_lock.manual_override = true;
        }
    }
}
