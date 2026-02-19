use std::collections::{HashMap, HashSet};

use bevy::{prelude::*, render::render_resource::TextureFormat};

use crate::{
    systems::{
        interaction::InteractionSystem,
        ui::layer::UiLayerKind,
    },
};

mod behavior;
mod geometry;
mod lifecycle;
mod scrollbar_math;
mod scrollbar;
#[cfg(test)]
mod tests;

use self::behavior::{
    handle_scrollable_pointer_and_keyboard_input, sync_scroll_content_offsets, sync_scroll_extents,
};
use self::lifecycle::{
    cleanup_scroll_layer_pool, ensure_scrollable_render_targets,
    ensure_scrollable_runtime_entities, sync_scroll_content_layers,
    sync_scrollable_render_entities, sync_scrollable_render_targets,
};
use self::scrollbar::{ensure_scrollbar_parts, handle_scrollbar_input, sync_scrollbar_visuals};
pub use self::geometry::cursor_in_edge_auto_scroll_zone;

const SCROLL_LAYER_START: u8 = 8;
const SCROLL_LAYER_COUNT: u8 = 20;
const SCROLL_SURFACE_Z: f32 = 0.02;
const SCROLL_CAMERA_Z: f32 = 10.0;
const SCROLL_WHEEL_LINE_PX: f32 = 36.0;
const SCROLL_KEYBOARD_STEP_PX: f32 = 40.0;
const SCROLL_PAGE_FACTOR: f32 = 0.92;
const SCROLL_EPSILON: f32 = 0.001;
const SCROLL_EDGE_ZONE_INSIDE_PX: f32 = 12.0;
const SCROLL_EDGE_ZONE_OUTSIDE_PX: f32 = 12.0;
const SCROLL_EDGE_AUTO_STEP_PX: f32 = 1.8;
const SCROLLBAR_BORDER_THICKNESS: f32 = 2.0;
const SCROLLBAR_THUMB_INSET: f32 = 2.0;
const SCROLLBAR_TRACK_FILL_COLOR: Color = Color::BLACK;
const SCROLLBAR_INTERACTION_Z: f32 = 1.45;
const SCROLLBAR_TRACK_Z: f32 = 0.01;
const SCROLLBAR_THUMB_Z: f32 = 0.02;
const SCROLLBAR_HITBOX_CROSS_AXIS_PAD_PX: f32 = 14.0;
const SCROLLBAR_HITBOX_MAIN_AXIS_PAD_PX: f32 = 6.0;

#[derive(Resource, Clone, Copy, Debug)]
pub struct ScrollRenderSettings {
    pub target_format: TextureFormat,
}

impl Default for ScrollRenderSettings {
    fn default() -> Self {
        Self {
            target_format: TextureFormat::Rgba16Float,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBackend {
    RenderToTexture,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
}

#[derive(Component, Clone, Copy, Debug)]
#[require(ScrollState, ScrollFocusFollowLock)]
pub struct ScrollableRoot {
    pub owner: Entity,
    pub axis: ScrollAxis,
    pub backend: ScrollBackend,
    pub input_layer: UiLayerKind,
}

impl ScrollableRoot {
    pub const fn new(owner: Entity, axis: ScrollAxis) -> Self {
        Self {
            owner,
            axis,
            backend: ScrollBackend::RenderToTexture,
            input_layer: UiLayerKind::Base,
        }
    }

    pub const fn with_input_layer(mut self, input_layer: UiLayerKind) -> Self {
        self.input_layer = input_layer;
        self
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollableViewport {
    pub size: Vec2,
}

impl ScrollableViewport {
    pub const fn new(size: Vec2) -> Self {
        Self { size }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ScrollState {
    pub offset_px: f32,
    pub content_extent: f32,
    pub viewport_extent: f32,
    pub max_offset: f32,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ScrollFocusFollowLock {
    pub manual_override: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollableContentExtent(pub f32);

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollableItem {
    pub key: u64,
    pub index: usize,
    pub extent: f32,
}

impl ScrollableItem {
    pub const fn new(key: u64, index: usize, extent: f32) -> Self {
        Self { key, index, extent }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollableContent;

#[derive(Component, Clone, Copy, Debug)]
struct ScrollableContentBaseTranslation(Vec3);

#[derive(Component, Clone, Debug)]
struct ScrollableRenderTarget {
    image: Handle<Image>,
    size_px: UVec2,
    layer: u8,
    format: TextureFormat,
}

#[derive(Component, Clone, Copy, Debug)]
struct ScrollableContentCamera {
    root: Entity,
}

#[derive(Component, Clone, Copy, Debug)]
struct ScrollableSurface {
    root: Entity,
}

#[derive(Component, Clone, Copy, Debug)]
struct ScrollLayerManaged;

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollBar {
    pub scrollable_root: Entity,
    pub width: f32,
    pub margin: f32,
    pub min_thumb_extent: f32,
    pub track_color: Color,
    pub thumb_color: Color,
}

impl ScrollBar {
    pub const fn new(scrollable_root: Entity) -> Self {
        Self {
            scrollable_root,
            width: 12.0,
            margin: 6.0,
            min_thumb_extent: 22.0,
            track_color: Color::srgb(0.2, 0.9, 0.2),
            thumb_color: Color::srgb(0.2, 0.9, 0.2),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
struct ScrollBarDragState {
    dragging: bool,
    grab_offset: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct ScrollBarParts {
    track: Entity,
    thumb: Entity,
}

#[derive(Component)]
struct ScrollBarTrack;

#[derive(Component)]
struct ScrollBarThumb;

#[derive(Resource, Debug)]
struct ScrollLayerPool {
    by_root: HashMap<Entity, u8>,
    free_layers: Vec<u8>,
}

impl Default for ScrollLayerPool {
    fn default() -> Self {
        let free_layers = (SCROLL_LAYER_START..SCROLL_LAYER_START + SCROLL_LAYER_COUNT).collect();
        Self {
            by_root: HashMap::new(),
            free_layers,
        }
    }
}

impl ScrollLayerPool {
    fn release_stale_roots(&mut self, live_roots: &HashSet<Entity>) {
        let mut released = Vec::new();
        self.by_root.retain(|root, layer| {
            if live_roots.contains(root) {
                true
            } else {
                released.push(*layer);
                false
            }
        });
        self.free_layers.extend(released);
        self.free_layers.sort_unstable();
        self.free_layers.dedup();
    }

    fn layer_for_root(&mut self, root: Entity) -> Option<u8> {
        if let Some(layer) = self.by_root.get(&root).copied() {
            return Some(layer);
        }

        self.free_layers.sort_unstable();
        let layer = self.free_layers.first().copied()?;
        self.free_layers.remove(0);
        self.by_root.insert(root, layer);
        Some(layer)
    }
}

pub struct ScrollPlugin;

impl Plugin for ScrollPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScrollLayerPool>()
            .init_resource::<ScrollRenderSettings>()
            .add_systems(
            Update,
            (
                cleanup_scroll_layer_pool,
                ensure_scrollable_render_targets,
                sync_scrollable_render_targets,
                ensure_scrollable_runtime_entities,
                sync_scrollable_render_entities,
                sync_scroll_content_layers,
                sync_scroll_extents,
                handle_scrollable_pointer_and_keyboard_input,
                sync_scroll_content_offsets,
                ensure_scrollbar_parts,
                sync_scrollbar_visuals,
                handle_scrollbar_input,
            )
                .chain()
                .after(InteractionSystem::Selectable),
        );
    }
}
