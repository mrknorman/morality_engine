use bevy::prelude::*;

use super::{
    ScrollAxis, SCROLL_EPSILON, SCROLLBAR_HITBOX_CROSS_AXIS_PAD_PX, SCROLLBAR_HITBOX_MAIN_AXIS_PAD_PX,
};

pub fn thumb_extent_for_state(
    track_extent: f32,
    viewport_extent: f32,
    content_extent: f32,
    min_thumb_extent: f32,
) -> f32 {
    let track_extent = track_extent.max(1.0);
    if content_extent <= SCROLL_EPSILON {
        return track_extent;
    }
    let ratio = (viewport_extent / content_extent).clamp(0.0, 1.0);
    (track_extent * ratio).clamp(min_thumb_extent.max(1.0), track_extent)
}

pub fn thumb_center_for_offset(
    track_extent: f32,
    thumb_extent: f32,
    offset: f32,
    max_offset: f32,
    axis: ScrollAxis,
) -> f32 {
    let track_extent = track_extent.max(1.0);
    let thumb_extent = thumb_extent.clamp(1.0, track_extent);
    let travel = (track_extent - thumb_extent).max(0.0);
    if travel <= SCROLL_EPSILON || max_offset <= SCROLL_EPSILON {
        return 0.0;
    }
    let normalized = (offset / max_offset).clamp(0.0, 1.0);

    match axis {
        ScrollAxis::Vertical => {
            let start = track_extent * 0.5 - thumb_extent * 0.5;
            start - normalized * travel
        }
        ScrollAxis::Horizontal => {
            let start = -track_extent * 0.5 + thumb_extent * 0.5;
            start + normalized * travel
        }
    }
}

pub fn offset_from_thumb_center(
    track_extent: f32,
    thumb_extent: f32,
    center: f32,
    max_offset: f32,
    axis: ScrollAxis,
) -> f32 {
    if max_offset <= SCROLL_EPSILON {
        return 0.0;
    }
    let track_extent = track_extent.max(1.0);
    let thumb_extent = thumb_extent.clamp(1.0, track_extent);
    let travel = (track_extent - thumb_extent).max(0.0);
    if travel <= SCROLL_EPSILON {
        return 0.0;
    }

    let normalized = match axis {
        ScrollAxis::Vertical => {
            let start = track_extent * 0.5 - thumb_extent * 0.5;
            ((start - center) / travel).clamp(0.0, 1.0)
        }
        ScrollAxis::Horizontal => {
            let start = -track_extent * 0.5 + thumb_extent * 0.5;
            ((center - start) / travel).clamp(0.0, 1.0)
        }
    };
    normalized * max_offset
}

pub(super) fn scrollbar_click_region(visual_size: Vec2, axis: ScrollAxis) -> Vec2 {
    match axis {
        ScrollAxis::Vertical => Vec2::new(
            visual_size.x + SCROLLBAR_HITBOX_CROSS_AXIS_PAD_PX,
            visual_size.y + SCROLLBAR_HITBOX_MAIN_AXIS_PAD_PX,
        ),
        ScrollAxis::Horizontal => Vec2::new(
            visual_size.x + SCROLLBAR_HITBOX_MAIN_AXIS_PAD_PX,
            visual_size.y + SCROLLBAR_HITBOX_CROSS_AXIS_PAD_PX,
        ),
    }
}
