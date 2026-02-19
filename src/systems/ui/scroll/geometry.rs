use bevy::prelude::*;

use super::{
    ScrollAxis, ScrollState, SCROLL_EDGE_AUTO_STEP_PX, SCROLL_EDGE_ZONE_INSIDE_PX,
    SCROLL_EDGE_ZONE_OUTSIDE_PX,
};

pub(super) fn axis_extent(size: Vec2, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Vertical => size.y,
        ScrollAxis::Horizontal => size.x,
    }
}

pub(super) fn axis_value_vec2(value: Vec2, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Vertical => value.y,
        ScrollAxis::Horizontal => value.x,
    }
}

pub(super) fn axis_value_vec3(value: Vec3, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Vertical => value.y,
        ScrollAxis::Horizontal => value.x,
    }
}

pub(super) fn edge_auto_scroll_delta(
    cursor_position: Vec2,
    global_transform: &GlobalTransform,
    viewport_size: Vec2,
    axis: ScrollAxis,
) -> Option<f32> {
    if axis != ScrollAxis::Vertical {
        return None;
    }

    let inverse = global_transform.to_matrix().inverse();
    let local = inverse.transform_point3(cursor_position.extend(0.0));
    let half_width = viewport_size.x * 0.5;
    let half_height = viewport_size.y * 0.5;
    if local.x.abs() > half_width + SCROLL_EDGE_ZONE_OUTSIDE_PX {
        return None;
    }

    let top_start = half_height - SCROLL_EDGE_ZONE_INSIDE_PX;
    let top_end = half_height + SCROLL_EDGE_ZONE_OUTSIDE_PX;
    if local.y >= top_start && local.y <= top_end {
        let denom = (top_end - top_start).max(1.0);
        let intensity = ((local.y - top_start) / denom).clamp(0.0, 1.0);
        return Some(-SCROLL_EDGE_AUTO_STEP_PX * intensity);
    }

    let bottom_start = -half_height + SCROLL_EDGE_ZONE_INSIDE_PX;
    let bottom_end = -half_height - SCROLL_EDGE_ZONE_OUTSIDE_PX;
    if local.y <= bottom_start && local.y >= bottom_end {
        let denom = (bottom_start - bottom_end).max(1.0);
        let intensity = ((bottom_start - local.y) / denom).clamp(0.0, 1.0);
        return Some(SCROLL_EDGE_AUTO_STEP_PX * intensity);
    }

    None
}

pub fn cursor_in_edge_auto_scroll_zone(
    cursor_position: Vec2,
    global_transform: &GlobalTransform,
    viewport_size: Vec2,
    axis: ScrollAxis,
) -> bool {
    edge_auto_scroll_delta(cursor_position, global_transform, viewport_size, axis).is_some()
}

pub(super) fn viewport_texture_size(size: Vec2) -> UVec2 {
    let width = size.x.round().max(1.0) as u32;
    let height = size.y.round().max(1.0) as u32;
    UVec2::new(width, height)
}

pub(super) fn clamp_scroll_state(state: &mut ScrollState) {
    state.content_extent = state.content_extent.max(0.0);
    state.viewport_extent = state.viewport_extent.max(0.0);
    state.max_offset = (state.content_extent - state.viewport_extent).max(0.0);
    state.offset_px = state.offset_px.clamp(0.0, state.max_offset);
}
