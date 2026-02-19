use std::collections::{HashMap, HashSet};

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::render_resource::TextureFormat,
};

use crate::{
    data::states::PauseState,
    entities::sprites::compound::{BorderedRectangle, HollowRectangle},
    startup::cursor::CustomCursor,
    systems::{
        interaction::{
            interaction_gate_allows_for_owner, is_cursor_within_region, Clickable,
            InteractionCapture, InteractionCaptureOwner, InteractionGate, InteractionSystem,
            ScrollUiActions,
        },
        ui::layer::{self, UiLayer, UiLayerKind},
    },
};

mod geometry;
mod lifecycle;
mod scrollbar_math;
#[cfg(test)]
mod tests;

use self::geometry::{
    axis_extent, axis_value_vec2, axis_value_vec3, clamp_scroll_state, edge_auto_scroll_delta,
};
use self::lifecycle::{
    cleanup_scroll_layer_pool, ensure_scrollable_render_targets,
    ensure_scrollable_runtime_entities, sync_scroll_content_layers,
    sync_scrollable_render_entities, sync_scrollable_render_targets,
};
use self::scrollbar_math::scrollbar_click_region;
pub use self::geometry::cursor_in_edge_auto_scroll_zone;
pub use self::scrollbar_math::{
    offset_from_thumb_center, thumb_center_for_offset, thumb_extent_for_state,
};

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

fn sync_scroll_extents(
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
            extent_by_content.get(&content_entity).copied().unwrap_or(viewport_extent)
        } else {
            viewport_extent
        };

        state.viewport_extent = viewport_extent;
        state.content_extent = content_extent.max(viewport_extent);
        clamp_scroll_state(&mut state);
    }
}

fn handle_scrollable_pointer_and_keyboard_input(
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

        let keyboard_candidate = (
            entity,
            global_transform.translation().z,
            entity.to_bits(),
        );
        match keyboard_target {
            None => keyboard_target = Some(keyboard_candidate),
            Some((_, current_z, current_rank))
                if keyboard_candidate.1 > current_z
                    || (keyboard_candidate.1 == current_z && keyboard_candidate.2 > current_rank) =>
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

        let candidate = (
            entity,
            global_transform.translation().z,
            entity.to_bits(),
        );
        match hovered_target {
            None => hovered_target = Some(candidate),
            Some((_, current_z, current_rank))
                if candidate.1 > current_z || (candidate.1 == current_z && candidate.2 > current_rank) =>
            {
                hovered_target = Some(candidate);
            }
            _ => {}
        }
    }

    let mut target_entity = None;
    let mut edge_delta = 0.0;
    let mut pointer_target_active = false;
    if let Some((entity, _, _, resolved_edge_delta)) = hovered_target
        .map(|(entity, z, rank)| (entity, z, rank, 0.0))
        .or(edge_target)
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

fn sync_scroll_content_offsets(
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

fn ensure_scrollbar_parts(
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
            ScrollBarDragState::default(),
        ));
    }
}

fn sync_scrollbar_visuals(
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    root_query: Query<
        (&ScrollableRoot, &ScrollableViewport, &ScrollState, Option<&InheritedVisibility>),
        With<ScrollableRoot>,
    >,
    mut scrollbar_query: Query<(
        &ScrollBar,
        &ScrollBarParts,
        &mut Transform,
        Option<&InheritedVisibility>,
    ), (Without<ScrollBarTrack>, Without<ScrollBarThumb>)>,
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
    for (scrollbar, parts, mut scrollbar_transform, scrollbar_visibility) in scrollbar_query.iter_mut() {
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
        let has_scroll_range = state.max_offset > SCROLL_EPSILON && state.content_extent > state.viewport_extent;
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
                scrollbar_transform.translation.x = viewport.size.x * 0.5 + scrollbar.margin + thickness * 0.5;
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

fn handle_scrollbar_input(
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
    mut scrollbar_query: Query<(
        Entity,
        &ScrollBar,
        &ScrollBarParts,
        &mut ScrollBarDragState,
        Option<&InheritedVisibility>,
    )>,
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
                drag_state.grab_offset = axis_value_vec2(cursor_position, root.axis) - thumb_center_world;
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
