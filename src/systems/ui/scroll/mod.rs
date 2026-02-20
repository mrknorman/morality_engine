//! Render-to-texture scroll primitives and scrollbar behavior.
//!
//! `ScrollableRoot` defines owner, axis, input-layer contract, and backend.
//! Runtime systems manage render targets/cameras/surfaces and keep pointer,
//! keyboard, and scrollbar input deterministic.
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    render::render_resource::TextureFormat,
};

use crate::systems::{
    interaction::{InteractionGate, InteractionSystem},
    ui::layer::UiLayerKind,
};

mod behavior;
mod geometry;
mod lifecycle;
mod scrollbar;
mod scrollbar_math;
#[cfg(test)]
mod tests;

use self::behavior::{
    handle_scrollable_pointer_and_keyboard_input, sync_scroll_content_offsets, sync_scroll_extents,
};
pub use self::geometry::cursor_in_edge_auto_scroll_zone;
use self::lifecycle::{
    cleanup_scroll_layer_pool, ensure_scrollable_render_targets,
    ensure_scrollable_runtime_entities, sync_scroll_content_layers,
    sync_scrollable_render_entities, sync_scrollable_render_targets,
};
use self::scrollbar::{ensure_scrollbar_parts, handle_scrollbar_input, sync_scrollbar_visuals};

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
    /// Render target format for scroll surfaces.
    pub target_format: TextureFormat,
    /// Maximum number of active RTT roots that can hold allocated targets.
    ///
    /// This is clamped to the internal scroll-layer pool size.
    pub max_render_targets: usize,
    /// Policy used when render-target budget is exhausted.
    pub exhaustion_policy: ScrollRenderExhaustionPolicy,
}

impl Default for ScrollRenderSettings {
    fn default() -> Self {
        Self {
            target_format: TextureFormat::Rgba16Float,
            max_render_targets: SCROLL_LAYER_COUNT as usize,
            exhaustion_policy: ScrollRenderExhaustionPolicy::WarnAndSkipRoot,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollRenderExhaustionPolicy {
    /// Log a warning and skip RTT setup for additional roots.
    WarnAndSkipRoot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBackend {
    /// Off-screen render-to-texture backend.
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
    /// Owner entity for layer arbitration.
    pub owner: Entity,
    /// Scroll axis for content and input mapping.
    pub axis: ScrollAxis,
    /// Rendering backend.
    pub backend: ScrollBackend,
    /// Active UI layer required for input acceptance.
    pub input_layer: UiLayerKind,
    /// Inside-edge auto-scroll zone in pixels.
    pub edge_zone_inside_px: f32,
    /// Outside-edge auto-scroll zone in pixels.
    pub edge_zone_outside_px: f32,
}

impl ScrollableRoot {
    pub const fn new(owner: Entity, axis: ScrollAxis) -> Self {
        Self {
            owner,
            axis,
            backend: ScrollBackend::RenderToTexture,
            input_layer: UiLayerKind::Base,
            edge_zone_inside_px: SCROLL_EDGE_ZONE_INSIDE_PX,
            edge_zone_outside_px: SCROLL_EDGE_ZONE_OUTSIDE_PX,
        }
    }

    pub const fn with_input_layer(mut self, input_layer: UiLayerKind) -> Self {
        self.input_layer = input_layer;
        self
    }

    pub const fn with_edge_zones(mut self, inside_px: f32, outside_px: f32) -> Self {
        self.edge_zone_inside_px = inside_px;
        self.edge_zone_outside_px = outside_px;
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

/// Reusable row-based table adapter metadata for scroll focus-follow and
/// viewport clipping helpers.
#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollableTableAdapter {
    pub owner: Entity,
    pub row_count: usize,
    pub row_extent: f32,
    pub leading_padding: f32,
}

impl ScrollableTableAdapter {
    pub const fn new(
        owner: Entity,
        row_count: usize,
        row_extent: f32,
        leading_padding: f32,
    ) -> Self {
        Self {
            owner,
            row_count,
            row_extent,
            leading_padding,
        }
    }
}

/// Reusable typed list adapter metadata for scroll integrations outside table
/// menus.
#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollableListAdapter<T: Send + Sync + 'static> {
    pub owner: Entity,
    pub item_count: usize,
    pub item_extent: f32,
    pub leading_padding: f32,
    marker: PhantomData<T>,
}

impl<T: Send + Sync + 'static> ScrollableListAdapter<T> {
    pub const fn new(
        owner: Entity,
        item_count: usize,
        item_extent: f32,
        leading_padding: f32,
    ) -> Self {
        Self {
            owner,
            item_count,
            item_extent,
            leading_padding,
            marker: PhantomData,
        }
    }
}

/// Returns `(row_top, row_bottom)` in content-space for `row_index`.
pub fn row_top_and_bottom(row_index: usize, row_extent: f32, leading_padding: f32) -> (f32, f32) {
    let row_extent = row_extent.max(1.0);
    let leading_padding = leading_padding.max(0.0);
    let row_top = leading_padding + row_index as f32 * row_extent;
    let row_bottom = row_top + row_extent;
    (row_top, row_bottom)
}

/// Returns whether a row overlaps the current viewport window.
pub fn row_visible_in_viewport(
    state: &ScrollState,
    row_index: usize,
    row_extent: f32,
    leading_padding: f32,
) -> bool {
    let (row_top, row_bottom) = row_top_and_bottom(row_index, row_extent, leading_padding);
    let viewport_top = state.offset_px;
    let viewport_bottom = state.offset_px + state.viewport_extent;
    row_bottom > viewport_top + 0.001 && row_top < viewport_bottom - 0.001
}

/// Mutates `state.offset_px` so `row_index` is visible in the viewport.
pub fn focus_scroll_offset_to_row(
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
#[require(Transform, Visibility, ScrollBarDragState)]
#[component(on_insert = ScrollBar::on_insert)]
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

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let Some(scrollbar) = world.entity(entity).get::<ScrollBar>().copied() else {
            return;
        };

        let parent_mismatch = world
            .entity(entity)
            .get::<ChildOf>()
            .is_none_or(|parent| parent.parent() != scrollbar.scrollable_root);
        if parent_mismatch {
            world
                .commands()
                .entity(scrollbar.scrollable_root)
                .add_child(entity);
        }

        if world.entity(entity).contains::<ScrollBarParts>() {
            return;
        }

        let gate = world
            .get_entity(scrollbar.scrollable_root)
            .ok()
            .and_then(|root| root.get::<InteractionGate>().copied())
            .unwrap_or_default();
        scrollbar::seed_scrollbar_parts(&mut world.commands(), entity, &scrollbar, gate);
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

    fn layer_for_root(&mut self, root: Entity, max_targets: usize) -> Option<u8> {
        if let Some(layer) = self.by_root.get(&root).copied() {
            return Some(layer);
        }
        if self.by_root.len() >= max_targets {
            return None;
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
