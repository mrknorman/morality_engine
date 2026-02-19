use std::collections::{HashMap, HashSet};

use bevy::{
    asset::RenderAssetUsages,
    camera::{visibility::RenderLayers, ClearColorConfig, RenderTarget},
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::Hdr,
    },
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
        app.init_resource::<ScrollLayerPool>().add_systems(
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

fn axis_extent(size: Vec2, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Vertical => size.y,
        ScrollAxis::Horizontal => size.x,
    }
}

fn axis_value_vec2(value: Vec2, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Vertical => value.y,
        ScrollAxis::Horizontal => value.x,
    }
}

fn axis_value_vec3(value: Vec3, axis: ScrollAxis) -> f32 {
    match axis {
        ScrollAxis::Vertical => value.y,
        ScrollAxis::Horizontal => value.x,
    }
}

fn edge_auto_scroll_delta(
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

fn viewport_texture_size(size: Vec2) -> UVec2 {
    let width = size.x.round().max(1.0) as u32;
    let height = size.y.round().max(1.0) as u32;
    UVec2::new(width, height)
}

fn create_scroll_target_image(size_px: UVec2) -> Image {
    let size = Extent3d {
        width: size_px.x,
        height: size_px.y,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0u8; 16],
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
        | TextureUsages::COPY_DST
        | TextureUsages::RENDER_ATTACHMENT;
    image
}

fn clamp_scroll_state(state: &mut ScrollState) {
    state.content_extent = state.content_extent.max(0.0);
    state.viewport_extent = state.viewport_extent.max(0.0);
    state.max_offset = (state.content_extent - state.viewport_extent).max(0.0);
    state.offset_px = state.offset_px.clamp(0.0, state.max_offset);
}

fn cleanup_scroll_layer_pool(
    mut layer_pool: ResMut<ScrollLayerPool>,
    root_query: Query<Entity, With<ScrollableRoot>>,
) {
    let live_roots: HashSet<Entity> = root_query.iter().collect();
    layer_pool.release_stale_roots(&live_roots);
}

fn ensure_scrollable_render_targets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layer_pool: ResMut<ScrollLayerPool>,
    root_query: Query<
        (Entity, &ScrollableRoot, &ScrollableViewport),
        (With<ScrollableRoot>, Without<ScrollableRenderTarget>),
    >,
) {
    let mut roots: Vec<(Entity, UVec2)> = root_query
        .iter()
        .filter_map(|(entity, root, viewport)| {
            matches!(root.backend, ScrollBackend::RenderToTexture)
                .then_some((entity, viewport_texture_size(viewport.size)))
        })
        .collect();
    roots.sort_by_key(|(entity, _)| entity.to_bits());

    for (root_entity, size_px) in roots {
        let Some(layer) = layer_pool.layer_for_root(root_entity) else {
            warn!(
                "No free scroll render layers available for root {:?}; skipping RTT setup",
                root_entity
            );
            continue;
        };
        let image = images.add(create_scroll_target_image(size_px));
        commands.entity(root_entity).insert(ScrollableRenderTarget {
            image,
            size_px,
            layer,
        });
    }
}

fn sync_scrollable_render_targets(
    mut images: ResMut<Assets<Image>>,
    mut root_query: Query<
        (&ScrollableRoot, &ScrollableViewport, &mut ScrollableRenderTarget),
        With<ScrollableRoot>,
    >,
) {
    for (root, viewport, mut render_target) in root_query.iter_mut() {
        if !matches!(root.backend, ScrollBackend::RenderToTexture) {
            continue;
        }
        let required_size = viewport_texture_size(viewport.size);
        if render_target.size_px == required_size {
            continue;
        }

        render_target.size_px = required_size;
        render_target.image = images.add(create_scroll_target_image(required_size));
    }
}

fn ensure_scrollable_runtime_entities(
    mut commands: Commands,
    root_query: Query<(Entity, &ScrollableRenderTarget, Option<&Children>), With<ScrollableRoot>>,
    camera_marker_query: Query<(), With<ScrollableContentCamera>>,
    surface_marker_query: Query<(), With<ScrollableSurface>>,
) {
    for (root_entity, render_target, children) in root_query.iter() {
        let mut has_camera = false;
        let mut has_surface = false;

        if let Some(children) = children {
            for child in children.iter() {
                if camera_marker_query.get(child).is_ok() {
                    has_camera = true;
                }
                if surface_marker_query.get(child).is_ok() {
                    has_surface = true;
                }
            }
        }

        if !has_camera {
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn((
                    Name::new("scrollable_content_camera"),
                    Camera2d,
                    ScrollableContentCamera { root: root_entity },
                    RenderLayers::layer(render_target.layer as usize),
                    Hdr,
                    Camera {
                        clear_color: ClearColorConfig::Custom(Color::NONE),
                        ..default()
                    },
                    RenderTarget::Image(render_target.image.clone().into()),
                    Transform::from_xyz(0.0, 0.0, SCROLL_CAMERA_Z),
                ));
            });
        }

        if !has_surface {
            commands.entity(root_entity).with_children(|parent| {
                parent.spawn((
                    Name::new("scrollable_surface"),
                    ScrollableSurface { root: root_entity },
                    Sprite::from_image(render_target.image.clone()),
                    Transform::from_xyz(0.0, 0.0, SCROLL_SURFACE_Z),
                ));
            });
        }
    }
}

fn sync_scrollable_render_entities(
    root_query: Query<(Entity, &ScrollableViewport, &ScrollableRenderTarget), With<ScrollableRoot>>,
    mut camera_query: Query<
        (
            &ScrollableContentCamera,
            &mut Camera,
            &mut RenderLayers,
            &mut RenderTarget,
        ),
    >,
    mut surface_query: Query<(&ScrollableSurface, &mut Sprite, &mut Transform, &mut Visibility)>,
) {
    let mut root_map = HashMap::new();
    for (root_entity, viewport, render_target) in root_query.iter() {
        root_map.insert(
            root_entity,
            (viewport.size, render_target.image.clone(), render_target.layer),
        );
    }

    for (marker, mut camera, mut layers, mut target) in camera_query.iter_mut() {
        let Some((_, image, layer)) = root_map.get(&marker.root) else {
            continue;
        };
        *layers = RenderLayers::layer(*layer as usize);
        *target = RenderTarget::Image(image.clone().into());
        camera.clear_color = ClearColorConfig::Custom(Color::NONE);
    }

    for (marker, mut sprite, mut transform, mut visibility) in surface_query.iter_mut() {
        let Some((size, image, _)) = root_map.get(&marker.root) else {
            continue;
        };
        if sprite.image != *image {
            sprite.image = image.clone();
        }
        sprite.custom_size = Some(*size);
        transform.translation.z = SCROLL_SURFACE_Z;
        *visibility = if size.x <= 0.0 || size.y <= 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
}

fn sync_scroll_content_layers(
    mut commands: Commands,
    root_query: Query<(Entity, &ScrollableRenderTarget), With<ScrollableRoot>>,
    content_query: Query<Entity, (With<ScrollableContent>, With<ChildOf>)>,
    content_parent_query: Query<&ChildOf, With<ScrollableContent>>,
    children_query: Query<&Children>,
    layer_query: Query<(Option<&RenderLayers>, Option<&ScrollLayerManaged>)>,
) {
    let layer_by_root: HashMap<Entity, u8> = root_query
        .iter()
        .map(|(entity, target)| (entity, target.layer))
        .collect();

    for content_entity in content_query.iter() {
        let Ok(parent) = content_parent_query.get(content_entity) else {
            continue;
        };
        let Some(layer) = layer_by_root.get(&parent.parent()).copied() else {
            continue;
        };
        let target_layers = RenderLayers::layer(layer as usize);
        let mut stack = vec![content_entity];
        while let Some(entity) = stack.pop() {
            let Ok((existing_layers, managed)) = layer_query.get(entity) else {
                continue;
            };
            let should_manage = entity == content_entity || managed.is_some() || existing_layers.is_none();
            if should_manage {
                let already_synced = managed.is_some_and(|_| {
                    existing_layers.is_some_and(|existing_layers| *existing_layers == target_layers)
                });
                if !already_synced {
                    commands
                        .entity(entity)
                        .insert((target_layers.clone(), ScrollLayerManaged));
                }
            }
            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    stack.push(child);
                }
            }
        }
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

fn scrollbar_click_region(visual_size: Vec2, axis: ScrollAxis) -> Vec2 {
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

#[cfg(test)]
mod tests {
    use bevy::{
        asset::Assets,
        camera::visibility::RenderLayers,
        input::mouse::{MouseScrollUnit, MouseWheel},
        prelude::*,
    };

    use crate::{
        startup::cursor::CustomCursor,
        systems::ui::layer::{UiLayer, UiLayerKind},
    };

    use super::{
        offset_from_thumb_center, thumb_center_for_offset, thumb_extent_for_state, ScrollAxis,
        ScrollPlugin, ScrollState, ScrollableContent, ScrollableContentExtent, ScrollableRoot, ScrollableViewport,
    };

    fn make_scroll_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<MouseWheel>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.init_resource::<CustomCursor>();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(ScrollPlugin);
        app
    }

    fn spawn_owner_and_scroll_root(
        app: &mut App,
        content_extent: f32,
        viewport_extent: f32,
    ) -> (Entity, Entity) {
        let owner = app.world_mut().spawn_empty().id();
        let root = app
            .world_mut()
            .spawn((
                ScrollableRoot::new(owner, ScrollAxis::Vertical),
                ScrollableViewport::new(Vec2::new(160.0, viewport_extent)),
                ScrollableContentExtent(content_extent),
                ScrollState {
                    offset_px: 0.0,
                    content_extent,
                    viewport_extent,
                    max_offset: (content_extent - viewport_extent).max(0.0),
                },
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();
        (owner, root)
    }

    fn write_wheel(app: &mut App, y: f32) {
        app.world_mut().write_message(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y,
            window: Entity::PLACEHOLDER,
        });
    }

    #[test]
    fn thumb_extent_respects_minimum_and_track_bounds() {
        let thumb = thumb_extent_for_state(120.0, 20.0, 300.0, 18.0);
        assert_eq!(thumb, 18.0);

        let thumb = thumb_extent_for_state(120.0, 500.0, 300.0, 18.0);
        assert_eq!(thumb, 120.0);
    }

    #[test]
    fn vertical_thumb_round_trip_maps_offset_consistently() {
        let track = 180.0;
        let thumb = 36.0;
        let max = 400.0;
        let offset = 215.0;
        let center = thumb_center_for_offset(track, thumb, offset, max, ScrollAxis::Vertical);
        let mapped = offset_from_thumb_center(track, thumb, center, max, ScrollAxis::Vertical);

        assert!((mapped - offset).abs() < 0.001);
    }

    #[test]
    fn horizontal_thumb_round_trip_maps_offset_consistently() {
        let track = 260.0;
        let thumb = 42.0;
        let max = 900.0;
        let offset = 510.0;
        let center = thumb_center_for_offset(track, thumb, offset, max, ScrollAxis::Horizontal);
        let mapped = offset_from_thumb_center(track, thumb, center, max, ScrollAxis::Horizontal);

        assert!((mapped - offset).abs() < 0.001);
    }

    #[test]
    fn thumb_center_is_clamped_by_scroll_range() {
        let top = thumb_center_for_offset(200.0, 40.0, 0.0, 100.0, ScrollAxis::Vertical);
        let bottom = thumb_center_for_offset(200.0, 40.0, 100.0, 100.0, ScrollAxis::Vertical);
        let beyond = thumb_center_for_offset(200.0, 40.0, 1000.0, 100.0, ScrollAxis::Vertical);

        assert!(top > bottom);
        assert_eq!(bottom, beyond);
    }

    #[test]
    fn scroll_plugin_update_is_query_safe() {
        let mut app = make_scroll_test_app();
        app.update();
    }

    #[test]
    fn base_layer_allows_scroll_input() {
        let mut app = make_scroll_test_app();
        let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
        app.world_mut().spawn((
            UiLayer::new(owner, UiLayerKind::Base),
            Visibility::Visible,
        ));
        app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
        write_wheel(&mut app, -1.0);
        app.update();

        let state = app.world().get::<ScrollState>(root).expect("scroll state");
        assert!(state.offset_px > 0.0);
    }

    #[test]
    fn dropdown_layer_blocks_scroll_input() {
        let mut app = make_scroll_test_app();
        let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
        app.world_mut().spawn((
            UiLayer::new(owner, UiLayerKind::Base),
            Visibility::Visible,
        ));
        app.world_mut().spawn((
            UiLayer::new(owner, UiLayerKind::Dropdown),
            Visibility::Visible,
        ));
        app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
        write_wheel(&mut app, -1.0);
        app.update();

        let state = app.world().get::<ScrollState>(root).expect("scroll state");
        assert_eq!(state.offset_px, 0.0);
    }

    #[test]
    fn mixed_keyboard_and_wheel_input_is_deterministic() {
        let mut app = make_scroll_test_app();
        let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
        app.world_mut().spawn((
            UiLayer::new(owner, UiLayerKind::Base),
            Visibility::Visible,
        ));
        app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);
        write_wheel(&mut app, -1.0);
        app.update();

        let state = app.world().get::<ScrollState>(root).expect("scroll state");
        // Line wheel is scaled by 36 px; ArrowDown adds 40 px.
        assert!((state.offset_px - 76.0).abs() < 0.001);
    }

    #[test]
    fn stress_layer_toggle_keeps_scroll_state_clamped_and_panics_free() {
        let mut app = make_scroll_test_app();
        let (owner, root) = spawn_owner_and_scroll_root(&mut app, 420.0, 100.0);
        let base = app
            .world_mut()
            .spawn((UiLayer::new(owner, UiLayerKind::Base), Visibility::Visible))
            .id();
        let dropdown = app
            .world_mut()
            .spawn((UiLayer::new(owner, UiLayerKind::Dropdown), Visibility::Hidden))
            .id();
        let modal = app
            .world_mut()
            .spawn((UiLayer::new(owner, UiLayerKind::Modal), Visibility::Hidden))
            .id();
        app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);

        for step in 0..48usize {
            let phase = step % 3;
            *app.world_mut()
                .get_mut::<Visibility>(base)
                .expect("base visibility") = if phase == 0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            *app.world_mut()
                .get_mut::<Visibility>(dropdown)
                .expect("dropdown visibility") = if phase == 1 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            *app.world_mut()
                .get_mut::<Visibility>(modal)
                .expect("modal visibility") = if phase == 2 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            {
                let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keyboard.reset_all();
                if phase == 0 && step % 2 == 0 {
                    keyboard.press(KeyCode::ArrowDown);
                }
            }
            write_wheel(&mut app, -1.0);

            let before = app
                .world()
                .get::<ScrollState>(root)
                .expect("scroll state")
                .offset_px;
            app.update();
            let state = *app.world().get::<ScrollState>(root).expect("scroll state");
            if phase != 0 {
                assert_eq!(state.offset_px, before);
            }
            assert!(state.offset_px >= 0.0);
            assert!(state.offset_px <= state.max_offset + 0.001);
        }
    }

    #[test]
    fn keyboard_scroll_works_without_cursor_position() {
        let mut app = make_scroll_test_app();
        let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
        app.world_mut().spawn((
            UiLayer::new(owner, UiLayerKind::Base),
            Visibility::Visible,
        ));
        app.world_mut().resource_mut::<CustomCursor>().position = None;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowDown);

        app.update();

        let state = app.world().get::<ScrollState>(root).expect("scroll state");
        assert!(state.offset_px > 0.0);
    }

    #[test]
    fn modal_layer_can_receive_scroll_input_when_active() {
        let mut app = make_scroll_test_app();
        let (owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
        let updated_root = app
            .world()
            .get::<ScrollableRoot>(root)
            .copied()
            .expect("scroll root")
            .with_input_layer(UiLayerKind::Modal);
        app.world_mut().entity_mut(root).insert(updated_root);
        app.world_mut().spawn((
            UiLayer::new(owner, UiLayerKind::Modal),
            Visibility::Visible,
        ));
        app.world_mut().resource_mut::<CustomCursor>().position = Some(Vec2::ZERO);
        write_wheel(&mut app, -1.0);
        app.update();

        let state = app.world().get::<ScrollState>(root).expect("scroll state");
        assert!(state.offset_px > 0.0);
    }

    #[test]
    fn explicit_child_render_layer_is_not_overridden_by_scroll_layer_sync() {
        let mut app = make_scroll_test_app();
        let (_owner, root) = spawn_owner_and_scroll_root(&mut app, 300.0, 100.0);
        let content = app.world_mut().spawn((ScrollableContent, Transform::default())).id();
        app.world_mut().entity_mut(root).add_child(content);

        let custom_layer_child = app
            .world_mut()
            .spawn((RenderLayers::layer(31), Transform::default()))
            .id();
        app.world_mut()
            .entity_mut(content)
            .add_child(custom_layer_child);

        app.update();

        let layers = app
            .world()
            .get::<RenderLayers>(custom_layer_child)
            .expect("custom child render layers");
        assert_eq!(*layers, RenderLayers::layer(31));
    }
}
