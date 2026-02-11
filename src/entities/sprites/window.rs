use std::{collections::HashMap, f32::consts::FRAC_PI_4};

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
    window::PrimaryWindow,
};
use enum_map::{enum_map, Enum};

use crate::{
    entities::sprites::compound::*,
    startup::{cursor::CustomCursor, render::MainCamera},
    systems::{
        audio::{TransientAudio, TransientAudioPallet},
        colors::{
            ColorAnchor, CLICKED_BUTTON, HOVERED_BUTTON,
            PRIMARY_COLOR,
        },
        interaction::{
            ActionPallet, Clickable, Draggable, DraggableRegion, DraggableViewportBounds,
            InputAction, InteractionVisualPalette, InteractionVisualState,
        },
    },
};

/* ─────────────────────────  PLUGIN  ───────────────────────── */

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowZStack>()
            .init_resource::<ActiveWindowInteraction>()
            .configure_sets(
                Update,
                (
                    WindowSystem::Input,
                    WindowSystem::Resolve.after(WindowSystem::Input),
                    WindowSystem::Layout.after(WindowSystem::Resolve),
                ),
            )
            .add_systems(
                Update,
                (
                    Window::assign_stack_order,
                    Window::cache_parts,
                    Window::enact_resize,
                )
                    .chain()
                    .in_set(WindowSystem::Input),
            )
            .add_systems(
                Update,
                (
                    Window::resolve_constraints,
                    Window::sync_root_drag_bounds,
                    Window::clamp_to_viewport,
                )
                    .chain()
                    .in_set(WindowSystem::Resolve),
            )
            .add_systems(
                Update,
                Window::update
                    .in_set(WindowSystem::Layout)
                    .before(CompoundSystem::Propagate),
            );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum WindowSystem {
    Input,
    Resolve,
    Layout,
}

#[derive(Resource, Default)]
struct WindowZStack {
    next_order: u32,
}

#[derive(Component)]
struct WindowBaseZ(f32);

const WINDOW_Z_STEP: f32 = 10.0;
const WINDOW_RESIZE_HANDLE_SIZE: f32 = 20.0;
const WINDOW_MIN_WIDTH: f32 = 60.0;
const WINDOW_MIN_HEIGHT: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResizeCorner {
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy)]
struct ActiveWindowResizeState {
    window_entity: Entity,
    corner: ResizeCorner,
    fixed_x_world: f32,
    fixed_top_y_world: f32,
}

#[derive(Component)]
pub struct WindowResizeInProgress;

#[derive(Clone, Copy)]
enum WindowInteraction {
    Resizing(ActiveWindowResizeState),
}

#[derive(Resource, Default)]
struct ActiveWindowInteraction {
    state: Option<WindowInteraction>,
}

/* ─────────────────────────  DATA  ───────────────────────── */

#[derive(Component, Clone)]
pub struct WindowTitle {
    pub text: String,
    pub padding: f32,
}
impl Default for WindowTitle {
    fn default() -> Self {
        Self {
            text: String::new(),
            padding: 20.0,
        }
    }
}

#[derive(Clone, Component)]
#[require(Transform, Visibility)]
#[component(on_insert = Window::on_insert)]
pub struct Window {
    pub boundary: HollowRectangle,
    pub title: Option<WindowTitle>,
    pub header_height: f32,
    pub has_close_button: bool,
    pub root_entity: Option<Entity>,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct WindowContentHost {
    pub window_entity: Entity,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct WindowContentMetrics {
    pub min_inner: Vec2,
    pub preferred_inner: Vec2,
    pub max_inner: Option<Vec2>,
}

impl WindowContentMetrics {
    pub fn from_min_inner(min_inner: Vec2) -> Self {
        let min_inner = min_inner.max(Vec2::ZERO);
        Self {
            min_inner,
            preferred_inner: min_inner,
            max_inner: None,
        }
    }
}

impl Default for WindowContentMetrics {
    fn default() -> Self {
        Self {
            min_inner: Vec2::ZERO,
            preferred_inner: Vec2::ZERO,
            max_inner: None,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(dead_code)]
pub enum WindowOverflowPolicy {
    #[default]
    ConstrainToContent,
    AllowOverflow,
    // Reserved for future clipping support; currently treated like ConstrainToContent.
    ClipReserved,
    // Reserved for future scrolling support; currently treated like ConstrainToContent.
    ScrollReserved,
}

impl WindowOverflowPolicy {
    fn enforce_content_constraints(self) -> bool {
        !matches!(self, Self::AllowOverflow)
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WindowContentRect {
    pub inner_size: Vec2,
}

#[derive(Component, Default)]
struct WindowParts {
    body: Option<Entity>,
    body_border: Option<Entity>,
    header: Option<Entity>,
    header_border: Option<Entity>,
    title: Option<Entity>,
    close_button: Option<Entity>,
    close_button_border: Option<Entity>,
    close_button_icon: Option<Entity>,
}
impl Default for Window {
    fn default() -> Self {
        Self {
            title: None,
            boundary: HollowRectangle::default(),
            header_height: 20.0,
            has_close_button: true,
            root_entity: None,
        }
    }
}

/* ─────────────────────────  UPDATE  ───────────────────────── */

impl Window {
    pub fn new(
        title: Option<WindowTitle>,
        boundary: HollowRectangle,
        header_height: f32,
        has_close_button: bool,
        root_entity: Option<Entity>,
    ) -> Self {
        Self {
            title,
            boundary,
            header_height,
            has_close_button,
            root_entity,
        }
    }

    fn assign_stack_order(
        mut commands: Commands,
        mut z_stack: ResMut<WindowZStack>,
        q_added_windows: Query<(Entity, &Window), Added<Window>>,
        q_window_base: Query<&WindowBaseZ>,
        mut q_transforms: Query<&mut Transform>,
    ) {
        for (window_entity, window) in &q_added_windows {
            let root_entity = window.root_entity.unwrap_or(window_entity);

            let Ok(mut root_transform) = q_transforms.get_mut(root_entity) else {
                continue;
            };

            let base_z = if let Ok(base) = q_window_base.get(root_entity) {
                base.0
            } else {
                let z = root_transform.translation.z;
                commands.entity(root_entity).insert(WindowBaseZ(z));
                z
            };

            root_transform.translation.z = base_z + z_stack.next_order as f32 * WINDOW_Z_STEP;
            z_stack.next_order += 1;
        }
    }

    fn cache_parts(
        mut windows: Query<(Entity, &mut WindowParts), With<Window>>,
        children_q: Query<&Children>,
        q_body: Query<(), With<WindowBody>>,
        q_header: Query<(), With<WindowHeader>>,
        q_close_button: Query<(), With<WindowCloseButton>>,
        q_bordered: Query<(), With<BorderedRectangle>>,
        q_title: Query<(), With<WindowTitle>>,
        q_close_border: Query<(), With<WindowCloseButtonBorder>>,
        q_close_icon: Query<(), With<WindowCloseButtonIcon>>,
    ) {
        fn first_child_with(
            parent: Entity,
            children_q: &Query<&Children>,
            predicate: &impl Fn(Entity) -> bool,
        ) -> Option<Entity> {
            let children = children_q.get(parent).ok()?;
            children.iter().find(|&child| predicate(child))
        }

        fn refresh_slot(
            slot: &mut Option<Entity>,
            parent: Entity,
            children_q: &Query<&Children>,
            predicate: &impl Fn(Entity) -> bool,
        ) {
            if slot.as_ref().is_some_and(|&entity| predicate(entity)) {
                return;
            }
            *slot = first_child_with(parent, children_q, predicate);
        }

        for (window_entity, mut parts) in &mut windows {
            refresh_slot(&mut parts.body, window_entity, &children_q, &|entity| {
                q_body.get(entity).is_ok()
            });
            refresh_slot(&mut parts.header, window_entity, &children_q, &|entity| {
                q_header.get(entity).is_ok()
            });
            refresh_slot(
                &mut parts.close_button,
                window_entity,
                &children_q,
                &|entity| q_close_button.get(entity).is_ok(),
            );

            if let Some(body) = parts.body {
                refresh_slot(&mut parts.body_border, body, &children_q, &|entity| {
                    q_bordered.get(entity).is_ok()
                });
            } else {
                parts.body_border = None;
            }
            if let Some(header) = parts.header {
                refresh_slot(&mut parts.header_border, header, &children_q, &|entity| {
                    q_bordered.get(entity).is_ok()
                });
                refresh_slot(&mut parts.title, header, &children_q, &|entity| {
                    q_title.get(entity).is_ok()
                });
            } else {
                parts.header_border = None;
                parts.title = None;
            }
            if let Some(close_button) = parts.close_button {
                refresh_slot(
                    &mut parts.close_button_border,
                    close_button,
                    &children_q,
                    &|entity| q_close_border.get(entity).is_ok(),
                );
                refresh_slot(
                    &mut parts.close_button_icon,
                    close_button,
                    &children_q,
                    &|entity| q_close_icon.get(entity).is_ok(),
                );
            } else {
                parts.close_button_border = None;
                parts.close_button_icon = None;
            }

            if parts
                .body
                .as_ref()
                .is_some_and(|&entity| q_body.get(entity).is_err())
            {
                parts.body = None;
            }
            if parts
                .header
                .as_ref()
                .is_some_and(|&entity| q_header.get(entity).is_err())
            {
                parts.header = None;
            }
            if parts
                .close_button
                .as_ref()
                .is_some_and(|&entity| q_close_button.get(entity).is_err())
            {
                parts.close_button = None;
            }
            if parts
                .body_border
                .as_ref()
                .is_some_and(|&entity| q_bordered.get(entity).is_err())
            {
                parts.body_border = None;
            }
            if parts
                .header_border
                .as_ref()
                .is_some_and(|&entity| q_bordered.get(entity).is_err())
            {
                parts.header_border = None;
            }
            if parts
                .title
                .as_ref()
                .is_some_and(|&entity| q_title.get(entity).is_err())
            {
                parts.title = None;
            }
            if parts
                .close_button_border
                .as_ref()
                .is_some_and(|&entity| q_close_border.get(entity).is_err())
            {
                parts.close_button_border = None;
            }
            if parts
                .close_button_icon
                .as_ref()
                .is_some_and(|&entity| q_close_icon.get(entity).is_err())
            {
                parts.close_button_icon = None;
            }
        }
    }

    fn enact_resize(
        mut commands: Commands,
        mouse_input: Res<ButtonInput<MouseButton>>,
        cursor: Res<CustomCursor>,
        mut active_interaction: ResMut<ActiveWindowInteraction>,
        parent_globals: Query<&GlobalTransform>,
        mut draggables: Query<&mut Draggable>,
        mut windows: ParamSet<(
            Query<(
                Entity,
                &Window,
                &GlobalTransform,
                Option<&ChildOf>,
            )>,
            Query<(
                Entity,
                &mut Window,
                &mut Transform,
                &GlobalTransform,
                Option<&ChildOf>,
                Option<&WindowContentMetrics>,
                Option<&WindowOverflowPolicy>,
            )>,
            Query<(&mut Transform, Option<&ChildOf>)>,
        )>,
    ) {
        let Some(cursor_position) = cursor.position else {
            if let Some(WindowInteraction::Resizing(state)) = active_interaction.state.take() {
                commands
                    .entity(state.window_entity)
                    .remove::<WindowResizeInProgress>();
            }
            return;
        };

        if !mouse_input.pressed(MouseButton::Left) {
            if let Some(WindowInteraction::Resizing(state)) = active_interaction.state.take() {
                commands
                    .entity(state.window_entity)
                    .remove::<WindowResizeInProgress>();
            }
        }

        if mouse_input.just_pressed(MouseButton::Left) {
            let mut top_window_z: Option<f32> = None;
            {
                for (_, window, global_transform, _) in windows.p0().iter() {
                    if !Self::is_cursor_over_window_surface(
                        cursor_position,
                        window,
                        global_transform,
                    ) {
                        continue;
                    }
                    let z = global_transform.translation().z;
                    if top_window_z.is_none_or(|current| z > current) {
                        top_window_z = Some(z);
                    }
                }
            }

            let mut candidate: Option<(Entity, ResizeCorner, f32, f32, f32)> = None;
            {
                for (entity, window, global_transform, _) in windows.p0().iter() {
                    let z = global_transform.translation().z;
                    if let Some(blocking_z) = top_window_z {
                        if z + 0.001 < blocking_z {
                            continue;
                        }
                    }

                    let half_w = window.boundary.dimensions.x * 0.5;
                    let half_h = window.boundary.dimensions.y * 0.5;
                    let mut corner_hit: Option<ResizeCorner> = None;

                    if Self::is_cursor_over_corner(
                        cursor_position,
                        window,
                        global_transform,
                        ResizeCorner::BottomLeft,
                    ) {
                        corner_hit = Some(ResizeCorner::BottomLeft);
                    }
                    if Self::is_cursor_over_corner(
                        cursor_position,
                        window,
                        global_transform,
                        ResizeCorner::BottomRight,
                    ) {
                        corner_hit = Some(ResizeCorner::BottomRight);
                    }

                    let Some(corner) = corner_hit else {
                        continue;
                    };

                    let replace = match candidate {
                        None => true,
                        Some((current_entity, _, _, _, current_z)) => {
                            z > current_z
                                || (z == current_z && entity.index() > current_entity.index())
                        }
                    };

                    if replace {
                        let center_world = global_transform.translation().truncate();
                        let fixed_x = match corner {
                            ResizeCorner::BottomLeft => center_world.x + half_w,
                            ResizeCorner::BottomRight => center_world.x - half_w,
                        };
                        let fixed_top_y = center_world.y + half_h + window.header_height;

                        candidate = Some((entity, corner, fixed_x, fixed_top_y, z));
                    }
                }
            }

            if let Some((entity, corner, fixed_x, fixed_top_y, _)) = candidate {
                commands.entity(entity).insert(WindowResizeInProgress);
                active_interaction.state =
                    Some(WindowInteraction::Resizing(ActiveWindowResizeState {
                        window_entity: entity,
                        corner,
                        fixed_x_world: fixed_x,
                        fixed_top_y_world: fixed_top_y,
                    }));
            }
        }

        let Some(WindowInteraction::Resizing(state)) = active_interaction.state else {
            return;
        };

        if !mouse_input.pressed(MouseButton::Left) {
            return;
        }

        let mut root_correction: Option<(Entity, Vec2)> = None;
        let mut drag_region_update: Option<(Entity, Vec2, f32, f32)> = None;

        {
            let mut writable_windows = windows.p1();
            let Ok((_, mut window, mut window_transform, window_global, parent, metrics, overflow_policy)) =
                writable_windows.get_mut(state.window_entity)
            else {
                commands
                    .entity(state.window_entity)
                    .remove::<WindowResizeInProgress>();
                active_interaction.state = None;
                return;
            };

            let min_inner = Self::min_inner_constraints(&window, metrics, overflow_policy);
            let max_inner = Self::max_inner_constraints(metrics, overflow_policy);

            let unclamped_width = match state.corner {
                ResizeCorner::BottomLeft => state.fixed_x_world - cursor_position.x,
                ResizeCorner::BottomRight => cursor_position.x - state.fixed_x_world,
            };
            let unclamped_height = state.fixed_top_y_world - window.header_height - cursor_position.y;
            let clamped_inner = Self::clamp_inner_size(
                Vec2::new(unclamped_width, unclamped_height),
                min_inner,
                max_inner,
            );
            let new_width = clamped_inner.x;
            let new_height = clamped_inner.y;

            window.boundary.dimensions = Vec2::new(new_width, new_height);

            let desired_center_world = Vec2::new(
                match state.corner {
                    ResizeCorner::BottomLeft => state.fixed_x_world - new_width * 0.5,
                    ResizeCorner::BottomRight => state.fixed_x_world + new_width * 0.5,
                },
                state.fixed_top_y_world - window.header_height - new_height * 0.5,
            );

            let desired_center_local =
                Self::cursor_to_parent_local(desired_center_world, parent, &parent_globals);

            let root_entity = window.root_entity.unwrap_or(state.window_entity);
            if root_entity == state.window_entity {
                window_transform.translation.x = desired_center_local.x;
                window_transform.translation.y = desired_center_local.y;
            } else {
                let correction_world = desired_center_world - window_global.translation().truncate();
                if correction_world.length_squared() > 0.000001 {
                    root_correction = Some((root_entity, correction_world));
                }
            }

            if let Some(root_entity) = window.root_entity {
                let edge_center_local = Vec2::new(
                    window_transform.translation.x,
                    window_transform.translation.y + (new_height + window.header_height) * 0.5,
                );
                drag_region_update = Some((
                    root_entity,
                    edge_center_local,
                    new_width,
                    window.header_height,
                ));
            }
        }

        if let Some((root_entity, correction_world)) = root_correction {
            if let Ok((mut root_transform, root_parent)) = windows.p2().get_mut(root_entity) {
                let correction_local =
                    Self::world_delta_to_parent_local(correction_world, root_parent, &parent_globals);
                root_transform.translation.x += correction_local.x;
                root_transform.translation.y += correction_local.y;
            }
        }

        if let Some((root_entity, edge_center_local, new_width, header_height)) = drag_region_update {
            if let Ok(mut draggable) = draggables.get_mut(root_entity) {
                let edge_padding = 10.0;
                draggable.region = Some(DraggableRegion {
                    region: Vec2::new(new_width + edge_padding, header_height + edge_padding),
                    offset: edge_center_local,
                });
            }
        }
    }

    fn resolve_constraints(
        mut commands: Commands,
        mut windows: ParamSet<(
            Query<(
                Entity,
                &Window,
                Option<&WindowContentMetrics>,
                Option<&WindowOverflowPolicy>,
                Option<&WindowContentRect>,
            )>,
            Query<&mut Window>,
            Query<&mut WindowContentRect>,
        )>,
    ) {
        let mut window_updates: Vec<(Entity, Vec2)> = Vec::new();
        let mut rect_updates: Vec<(Entity, Vec2)> = Vec::new();
        let mut rect_inserts: Vec<(Entity, Vec2)> = Vec::new();

        {
            for (entity, window, metrics, overflow_policy, content_rect) in windows.p0().iter() {
                let min_inner = Self::min_inner_constraints(window, metrics, overflow_policy);
                let max_inner = Self::max_inner_constraints(metrics, overflow_policy);
                let clamped_inner =
                    Self::clamp_inner_size(window.boundary.dimensions, min_inner, max_inner);

                if (clamped_inner - window.boundary.dimensions).length_squared() > 0.0001 {
                    window_updates.push((entity, clamped_inner));
                }

                if let Some(content_rect) = content_rect {
                    if (content_rect.inner_size - clamped_inner).length_squared() > 0.0001 {
                        rect_updates.push((entity, clamped_inner));
                    }
                } else {
                    rect_inserts.push((entity, clamped_inner));
                }
            }
        }

        for (entity, clamped_inner) in window_updates {
            if let Ok(mut writable_window) = windows.p1().get_mut(entity) {
                writable_window.boundary.dimensions = clamped_inner;
            }
        }

        for (entity, clamped_inner) in rect_updates {
            if let Ok(mut writable_rect) = windows.p2().get_mut(entity) {
                writable_rect.inner_size = clamped_inner;
            }
        }

        for (entity, clamped_inner) in rect_inserts {
            commands.entity(entity).insert(WindowContentRect {
                inner_size: clamped_inner,
            });
        }
    }

    fn sync_root_drag_bounds(
        mut commands: Commands,
        window: Single<&bevy::window::Window, With<PrimaryWindow>>,
        camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
        windows: Query<(Entity, &Window, &GlobalTransform)>,
        root_globals: Query<&GlobalTransform>,
        existing_bounds: Query<Entity, With<DraggableViewportBounds>>,
    ) {
        let (camera, camera_transform) = *camera_query;
        let Some((viewport_min, viewport_max)) =
            Self::viewport_world_bounds(*window, camera, camera_transform)
        else {
            return;
        };

        let mut bounds_by_root: HashMap<Entity, DraggableViewportBounds> = HashMap::new();
        for (window_entity, window, window_global) in windows.iter() {
            let root_entity = window.root_entity.unwrap_or(window_entity);
            let Ok(root_global) = root_globals.get(root_entity) else {
                continue;
            };

            let window_center = window_global.translation().truncate();
            let root_center = root_global.translation().truncate();
            let window_offset_from_root = window_center - root_center;

            let half_w = window.boundary.dimensions.x * 0.5;
            let half_h = window.boundary.dimensions.y * 0.5;

            let min_window_center = Vec2::new(
                viewport_min.x + half_w,
                viewport_min.y + half_h,
            );
            let max_window_center = Vec2::new(
                viewport_max.x - half_w,
                viewport_max.y - (half_h + window.header_height),
            );
            let bounds = DraggableViewportBounds {
                min: min_window_center - window_offset_from_root,
                max: max_window_center - window_offset_from_root,
            };

            bounds_by_root
                .entry(root_entity)
                .and_modify(|aggregate| {
                    aggregate.min = aggregate.min.max(bounds.min);
                    aggregate.max = aggregate.max.min(bounds.max);
                })
                .or_insert(bounds);
        }

        for (root_entity, bounds) in bounds_by_root.iter() {
            commands.entity(*root_entity).insert(*bounds);
        }

        for entity in existing_bounds.iter() {
            if !bounds_by_root.contains_key(&entity) {
                commands.entity(entity).remove::<DraggableViewportBounds>();
            }
        }
    }

    fn clamp_to_viewport(
        window: Single<&bevy::window::Window, With<PrimaryWindow>>,
        camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
        windows: Query<(Entity, &Window, &GlobalTransform)>,
        mut root_transforms: Query<(&mut Transform, Option<&ChildOf>)>,
        parent_globals: Query<&GlobalTransform>,
    ) {
        let (camera, camera_transform) = *camera_query;
        let Some((viewport_min, viewport_max)) =
            Self::viewport_world_bounds(*window, camera, camera_transform)
        else {
            return;
        };

        let mut root_corrections: HashMap<Entity, Vec2> = HashMap::new();
        for (window_entity, window, window_global) in windows.iter() {
            let window_center = window_global.translation().truncate();
            let clamped_center = Self::clamp_window_center_to_bounds(
                window_center,
                window,
                viewport_min,
                viewport_max,
            );
            let correction_world = clamped_center - window_center;
            if correction_world.length_squared() <= 0.000001 {
                continue;
            }

            let root_entity = window.root_entity.unwrap_or(window_entity);
            root_corrections
                .entry(root_entity)
                .and_modify(|aggregate| {
                    if correction_world.x.abs() > aggregate.x.abs() {
                        aggregate.x = correction_world.x;
                    }
                    if correction_world.y.abs() > aggregate.y.abs() {
                        aggregate.y = correction_world.y;
                    }
                })
                .or_insert(correction_world);
        }

        for (root_entity, correction_world) in root_corrections {
            let Ok((mut root_transform, parent)) = root_transforms.get_mut(root_entity) else {
                continue;
            };
            let correction_local =
                Self::world_delta_to_parent_local(correction_world, parent, &parent_globals);
            root_transform.translation.x += correction_local.x;
            root_transform.translation.y += correction_local.y;
        }
    }

    fn viewport_world_bounds(
        window: &bevy::window::Window,
        camera: &Camera,
        camera_transform: &GlobalTransform,
    ) -> Option<(Vec2, Vec2)> {
        let size = Vec2::new(window.resolution.width(), window.resolution.height());
        let corners = [
            Vec2::new(0.0, 0.0),
            Vec2::new(size.x, 0.0),
            Vec2::new(0.0, size.y),
            Vec2::new(size.x, size.y),
        ];

        let mut world_points = Vec::with_capacity(corners.len());
        for corner in corners {
            let world = camera
                .viewport_to_world_2d(camera_transform, corner)
                .ok()?;
            world_points.push(world);
        }

        let mut min = world_points[0];
        let mut max = world_points[0];
        for point in world_points.into_iter().skip(1) {
            min.x = min.x.min(point.x);
            min.y = min.y.min(point.y);
            max.x = max.x.max(point.x);
            max.y = max.y.max(point.y);
        }
        Some((min, max))
    }

    fn clamp_window_center_to_bounds(
        window_center: Vec2,
        window: &Window,
        viewport_min: Vec2,
        viewport_max: Vec2,
    ) -> Vec2 {
        let half_w = window.boundary.dimensions.x * 0.5;
        let half_h = window.boundary.dimensions.y * 0.5;

        let min_center_x = viewport_min.x + half_w;
        let max_center_x = viewport_max.x - half_w;

        let min_center_y = viewport_min.y + half_h;
        let max_center_y = viewport_max.y - (half_h + window.header_height);

        let clamped_x = if min_center_x <= max_center_x {
            window_center.x.clamp(min_center_x, max_center_x)
        } else {
            (min_center_x + max_center_x) * 0.5
        };
        let clamped_y = if min_center_y <= max_center_y {
            window_center.y.clamp(min_center_y, max_center_y)
        } else {
            (min_center_y + max_center_y) * 0.5
        };

        Vec2::new(clamped_x, clamped_y)
    }

    fn world_delta_to_parent_local(
        world_delta: Vec2,
        parent: Option<&ChildOf>,
        parent_globals: &Query<&GlobalTransform>,
    ) -> Vec2 {
        if let Some(parent) = parent {
            if let Ok(parent_global) = parent_globals.get(parent.parent()) {
                let local_delta = parent_global
                    .to_matrix()
                    .inverse()
                    .transform_vector3(world_delta.extend(0.0))
                    .truncate();
                return local_delta;
            }
        }
        world_delta
    }

    fn clamp_inner_size(size: Vec2, min_inner: Vec2, max_inner: Option<Vec2>) -> Vec2 {
        let mut clamped = size.max(min_inner.max(Vec2::ZERO));
        if let Some(max_inner) = max_inner {
            let max_inner = max_inner.max(min_inner);
            clamped = clamped.min(max_inner);
        }
        clamped
    }

    fn min_inner_constraints(
        window: &Window,
        metrics: Option<&WindowContentMetrics>,
        overflow_policy: Option<&WindowOverflowPolicy>,
    ) -> Vec2 {
        let mut min_inner = Vec2::new(
            WINDOW_MIN_WIDTH.max(window.header_height + 10.0),
            WINDOW_MIN_HEIGHT,
        );
        let overflow_policy = overflow_policy.copied().unwrap_or_default();
        if overflow_policy.enforce_content_constraints() {
            if let Some(metrics) = metrics {
                min_inner = min_inner.max(metrics.min_inner.max(Vec2::ZERO));
            }
        }
        min_inner
    }

    fn max_inner_constraints(
        metrics: Option<&WindowContentMetrics>,
        overflow_policy: Option<&WindowOverflowPolicy>,
    ) -> Option<Vec2> {
        let overflow_policy = overflow_policy.copied().unwrap_or_default();
        if !overflow_policy.enforce_content_constraints() {
            return None;
        }
        metrics.and_then(|metrics| metrics.max_inner)
    }

    fn cursor_to_parent_local(
        cursor_world: Vec2,
        parent: Option<&ChildOf>,
        parent_globals: &Query<&GlobalTransform>,
    ) -> Vec2 {
        if let Some(parent) = parent {
            if let Ok(parent_global) = parent_globals.get(parent.parent()) {
                let cursor_local = parent_global
                    .to_matrix()
                    .inverse()
                    .transform_point3(cursor_world.extend(0.0));
                return cursor_local.truncate();
            }
        }
        cursor_world
    }

    fn is_cursor_over_corner(
        cursor_world: Vec2,
        window: &Window,
        window_global: &GlobalTransform,
        corner: ResizeCorner,
    ) -> bool {
        let cursor_local = window_global
            .to_matrix()
            .inverse()
            .transform_point3(cursor_world.extend(0.0))
            .truncate();
        let half_w = window.boundary.dimensions.x * 0.5;
        let half_h = window.boundary.dimensions.y * 0.5;
        let corner_local = match corner {
            ResizeCorner::BottomLeft => Vec2::new(-half_w, -half_h),
            ResizeCorner::BottomRight => Vec2::new(half_w, -half_h),
        };
        let delta = cursor_local - corner_local;
        delta.x.abs() <= WINDOW_RESIZE_HANDLE_SIZE * 0.5
            && delta.y.abs() <= WINDOW_RESIZE_HANDLE_SIZE * 0.5
    }

    fn is_cursor_over_window_surface(
        cursor_world: Vec2,
        window: &Window,
        window_global: &GlobalTransform,
    ) -> bool {
        let cursor_local = window_global
            .to_matrix()
            .inverse()
            .transform_point3(cursor_world.extend(0.0))
            .truncate();
        let region_center = Vec2::new(0.0, window.header_height * 0.5);
        let half_extents = Vec2::new(
            window.boundary.dimensions.x * 0.5,
            (window.boundary.dimensions.y + window.header_height) * 0.5,
        );
        let delta = cursor_local - region_center;
        delta.x.abs() <= half_extents.x && delta.y.abs() <= half_extents.y
    }

    fn spawn_title_child(
        commands: &mut Commands,
        header_entity: Entity,
        title: &WindowTitle,
        header_width: f32,
    ) -> Option<Entity> {
        let mut title_entity = None;
        commands.entity(header_entity).with_children(|parent| {
            title_entity = Some(
                parent
                    .spawn((
                        title.clone(),
                        Text2d(title.text.clone()),
                        TextColor(PRIMARY_COLOR),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        Anchor::CENTER_LEFT,
                        Transform::from_xyz((-header_width + title.padding) / 2.0, 0.0, 0.0),
                    ))
                    .id(),
            );
        });
        title_entity
    }

    fn spawn_close_button_child(
        commands: &mut Commands,
        window_entity: Entity,
        window: &Window,
    ) -> Option<Entity> {
        let mut close_entity = None;
        commands.entity(window_entity).with_children(|parent| {
            close_entity = Some(
                parent
                    .spawn((
                        WindowCloseButton {
                            root_entity: window.root_entity,
                        },
                        Transform::from_xyz(
                            (window.boundary.dimensions.x - window.header_height) / 2.0,
                            (window.boundary.dimensions.y + window.header_height) / 2.0,
                            0.0,
                        ),
                    ))
                    .id(),
            );
        });
        close_entity
    }

    /// Propagate any change on the Window to **all** its descendants.
    fn update(
        mut commands: Commands,
        mut windows: Query<(Entity, &Window, &mut WindowParts), Changed<Window>>,
        existing_entities: Query<(), ()>,
        mut borders: Query<&mut BorderedRectangle>,
        mut plus_sets: Query<(
            &mut Plus,
            &mut Clickable<WindowActions>,
            &mut ColorAnchor,
            &mut InteractionVisualPalette,
        )>,
        mut header_titles: Query<
            (&mut WindowTitle, &mut Text2d, &mut Transform),
            (Without<WindowHeader>, Without<WindowCloseButton>),
        >,
        mut hb: ParamSet<(
            Query<&mut Transform, With<WindowHeader>>,
            Query<&mut Transform, With<WindowCloseButton>>,
        )>,
    ) {
        for (window_entity, win, mut parts) in &mut windows {
            if let Some(body_border_entity) = parts.body_border {
                if let Ok(mut border) = borders.get_mut(body_border_entity) {
                    border.boundary = win.boundary;
                }
            }

            if let Some(header_entity) = parts.header {
                let header_boundary = HollowRectangle {
                    dimensions: Vec2::new(win.boundary.dimensions.x, win.header_height),
                    ..win.boundary
                };
                if let Ok(mut tf) = hb.p0().get_mut(header_entity) {
                    tf.translation = Vec3::new(
                        0.0,
                        (win.boundary.dimensions.y + win.header_height) / 2.0,
                        0.0,
                    );
                }

                if let Some(header_border_entity) = parts.header_border {
                    if let Ok(mut border) = borders.get_mut(header_border_entity) {
                        border.boundary = header_boundary;
                    }
                }

                match (&win.title, parts.title) {
                    (Some(new_title), Some(title_entity)) => {
                        if let Ok((mut title, mut text, mut title_tf)) =
                            header_titles.get_mut(title_entity)
                        {
                            *title = new_title.clone();
                            text.0 = new_title.text.clone();
                            title_tf.translation = Vec3::new(
                                (-header_boundary.dimensions.x + new_title.padding) / 2.0,
                                0.0,
                                0.0,
                            );
                        } else {
                            parts.title = None;
                        }
                    }
                    (Some(new_title), None) => {
                        parts.title = Self::spawn_title_child(
                            &mut commands,
                            header_entity,
                            new_title,
                            header_boundary.dimensions.x,
                        );
                    }
                    (None, Some(title_entity)) => {
                        if existing_entities.get(title_entity).is_ok() {
                            commands.entity(title_entity).despawn();
                        }
                        parts.title = None;
                    }
                    (None, None) => {}
                }
            }

            if win.has_close_button {
                if parts.close_button.is_none() {
                    parts.close_button =
                        Self::spawn_close_button_child(&mut commands, window_entity, win);
                    parts.close_button_border = None;
                    parts.close_button_icon = None;
                }
            } else if let Some(close_button_entity) = parts.close_button.take() {
                if existing_entities.get(close_button_entity).is_ok() {
                    commands.entity(close_button_entity).despawn();
                }
                parts.close_button_border = None;
                parts.close_button_icon = None;
            }

            if let Some(close_button_entity) = parts.close_button {
                let close_boundary = HollowRectangle {
                    dimensions: Vec2::splat(win.header_height),
                    thickness: win.boundary.thickness,
                    color: win.boundary.color,
                    ..default()
                };
                if let Ok(mut tf) = hb.p1().get_mut(close_button_entity) {
                    tf.translation = Vec3::new(
                        (win.boundary.dimensions.x - win.header_height) / 2.0,
                        (win.boundary.dimensions.y + win.header_height) / 2.0,
                        0.0,
                    );
                }

                if let Some(close_border_entity) = parts.close_button_border {
                    if let Ok(mut border) = borders.get_mut(close_border_entity) {
                        border.boundary = close_boundary;
                    }
                }
                if let Some(close_icon_entity) = parts.close_button_icon {
                    if let Ok((mut plus, mut click, mut color_anchor, mut palette)) =
                        plus_sets.get_mut(close_icon_entity)
                    {
                        plus.dimensions = close_boundary.dimensions - 10.0;
                        plus.color = close_boundary.color;
                        let region = Some(close_boundary.dimensions + 10.0);
                        click.region = region;
                        color_anchor.0 = close_boundary.color;
                        palette.idle_color = close_boundary.color;
                    }
                }
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (boundary, header_h, close_btn, root_entity) = {
            let mut entity_mut = world.entity_mut(entity);
            let mut w = entity_mut.get_mut::<Window>().unwrap();
            // ensure root is self if not set
            if w.root_entity.is_none() {
                w.root_entity = Some(entity);
            }
            (
                w.boundary,
                w.header_height,
                w.has_close_button,
                w.root_entity,
            )
        };

        let mut body_entity: Option<Entity> = None;
        let mut header_entity: Option<Entity> = None;
        let mut close_button_entity: Option<Entity> = None;

        /*  Spawn children *directly* under Window  */
        world.commands().entity(entity).with_children(|parent| {
            // Body -------------
            body_entity = Some(parent.spawn(WindowBody).id());

            // Header -----------
            header_entity = Some(
                parent
                    .spawn((
                        WindowHeader,
                        Transform::from_xyz(0.0, (boundary.dimensions.y + header_h) / 2.0, 0.0),
                    ))
                    .id(),
            );

            // Close button -----
            if close_btn {
                close_button_entity = Some(
                    parent
                        .spawn((
                            WindowCloseButton { root_entity },
                            Transform::from_xyz(
                                (boundary.dimensions.x - header_h) / 2.0,
                                (boundary.dimensions.y + header_h) / 2.0,
                                0.0,
                            ),
                        ))
                        .id(),
                );
            }
        });

        world.commands().entity(entity).insert(WindowParts {
            body: body_entity,
            header: header_entity,
            close_button: close_button_entity,
            ..default()
        });
    }
}

#[derive(Clone, Component)]
#[component(on_insert = WindowBody::on_insert)]
#[require(Transform, Visibility)]
pub struct WindowBody;

impl WindowBody {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        world.commands().entity(entity).with_children(|p| {
            p.spawn(BorderedRectangle::default());
        });
    }
}

/* ─────────────────────────  HEADER  ───────────────────────── */

#[derive(Clone, Component)]
#[component(on_insert = WindowHeader::on_insert)]
#[require(Transform, Visibility)]
pub struct WindowHeader;

impl WindowHeader {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        world.commands().entity(entity).with_children(|parent| {
            parent.spawn(BorderedRectangle::default());
        });
    }
}

#[derive(Clone, Component)]
#[component(on_insert = WindowCloseButton::on_insert)]
#[require(Transform, Visibility)]
pub struct WindowCloseButton {
    pub root_entity: Option<Entity>,
}

impl WindowCloseButton {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let root = {
            let b = world.entity(entity).get::<WindowCloseButton>().unwrap();
            b.root_entity
        };

        let handle = world
            .get_resource::<AssetServer>()
            .unwrap()
            .load("./audio/effects/mouse_click.ogg");

        world.commands().entity(entity).with_children(|parent| {
            let default_region = Vec2::splat(20.0);
            let default_icon_dim = Vec2::splat(10.0);
            parent.spawn((WindowCloseButtonBorder, BorderedRectangle::default()));

            parent.spawn((
                WindowCloseButtonIcon,
                Plus {
                    dimensions: default_icon_dim,
                    thickness: 1.0,
                    color: PRIMARY_COLOR,
                },
                Clickable::with_region(vec![WindowActions::CloseWindow], default_region),
                InteractionVisualState::default(),
                InteractionVisualPalette::new(
                    PRIMARY_COLOR,
                    HOVERED_BUTTON,
                    CLICKED_BUTTON,
                    HOVERED_BUTTON,
                ),
                ColorAnchor(PRIMARY_COLOR),
                Transform {
                    rotation: Quat::from_rotation_z(FRAC_PI_4),
                    ..default()
                },
                ActionPallet::<WindowActions, WindowSounds>(enum_map!(
                    WindowActions::CloseWindow => vec![
                        InputAction::PlaySound(WindowSounds::Close),
                        InputAction::Despawn(root)
                    ]
                )),
                TransientAudioPallet::new(vec![(
                    WindowSounds::Close,
                    vec![TransientAudio::new(handle, 0.1, true, 1.0, true)],
                )]),
            ));
        });
    }
}

#[derive(Component)]
struct WindowCloseButtonBorder;
#[derive(Component)]
struct WindowCloseButtonIcon;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSounds {
    Close,
}
#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowActions {
    CloseWindow,
}
