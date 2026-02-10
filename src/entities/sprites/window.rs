use std::f32::consts::FRAC_PI_4;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
};
use enum_map::{enum_map, Enum};

use crate::{
    entities::sprites::compound::*,
    systems::{
        audio::{TransientAudio, TransientAudioPallet},
        colors::{
            ColorAnchor, ColorChangeEvent, ColorChangeOn, CLICKED_BUTTON, HOVERED_BUTTON,
            PRIMARY_COLOR,
        },
        interaction::{ActionPallet, Clickable, Draggable, DraggableRegion, InputAction},
    },
    startup::cursor::CustomCursor,
};

/* ─────────────────────────  PLUGIN  ───────────────────────── */

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<WindowZStack>()
            .init_resource::<ActiveWindowResize>()
            .add_systems(Update, (
                Window::assign_stack_order,
                Window::enact_resize,
                Window::update.before(CompoundSystem::Propagate),
            ).chain());
    }
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
    fixed_x: f32,
    fixed_top_y: f32,
}

#[derive(Resource, Default)]
struct ActiveWindowResize {
    state: Option<ActiveWindowResizeState>,
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
        title : Option<WindowTitle>,
        boundary : HollowRectangle,
        header_height : f32,
        has_close_button : bool,
        root_entity : Option<Entity>
    ) -> Self {
        Self{
            title,
            boundary,
            header_height,
            has_close_button,
            root_entity
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

    fn enact_resize(
        mouse_input: Res<ButtonInput<MouseButton>>,
        cursor: Res<CustomCursor>,
        mut active_resize: ResMut<ActiveWindowResize>,
        parent_globals: Query<&GlobalTransform>,
        mut draggables: Query<&mut Draggable>,
        mut windows: ParamSet<(
            Query<
                (
                    Entity,
                    &Window,
                    &Transform,
                    &GlobalTransform,
                    Option<&ChildOf>,
                )
            >,
            Query<(Entity, &mut Window, &mut Transform, Option<&ChildOf>)>,
        )>,
    ) {
        let Some(cursor_position) = cursor.position else {
            active_resize.state = None;
            return;
        };

        if !mouse_input.pressed(MouseButton::Left) {
            active_resize.state = None;
        }

        if mouse_input.just_pressed(MouseButton::Left) {
            let mut top_window_z: Option<f32> = None;
            {
                for (_, window, _, global_transform, _) in windows.p0().iter() {
                    if !Self::is_cursor_over_window_surface(cursor_position, window, global_transform)
                    {
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
                for (entity, window, transform, global_transform, parent) in windows.p0().iter() {
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
                            z > current_z || (z == current_z && entity.index() > current_entity.index())
                        }
                    };

                    if replace {
                        let fixed_x = match corner {
                            ResizeCorner::BottomLeft => transform.translation.x + half_w,
                            ResizeCorner::BottomRight => transform.translation.x - half_w,
                        };
                        let fixed_top_y =
                            transform.translation.y + half_h + window.header_height;

                        let _ = parent; // parent transform is only needed during drag updates
                        candidate = Some((entity, corner, fixed_x, fixed_top_y, z));
                    }
                }
            }

            if let Some((entity, corner, fixed_x, fixed_top_y, _)) = candidate {
                active_resize.state = Some(ActiveWindowResizeState {
                    window_entity: entity,
                    corner,
                    fixed_x,
                    fixed_top_y,
                });
            }
        }

        let Some(state) = active_resize.state else {
            return;
        };

        if !mouse_input.pressed(MouseButton::Left) {
            return;
        }

        let mut writable_windows = windows.p1();
        let Ok((_, mut window, mut window_transform, parent)) =
            writable_windows.get_mut(state.window_entity)
        else {
            active_resize.state = None;
            return;
        };

        let cursor_local_parent =
            Self::cursor_to_parent_local(cursor_position, parent, &parent_globals);

        let min_width = WINDOW_MIN_WIDTH.max(window.header_height + 10.0);
        let min_height = WINDOW_MIN_HEIGHT;

        let new_width = match state.corner {
            ResizeCorner::BottomLeft => (state.fixed_x - cursor_local_parent.x).max(min_width),
            ResizeCorner::BottomRight => (cursor_local_parent.x - state.fixed_x).max(min_width),
        };
        let new_height =
            (state.fixed_top_y - window.header_height - cursor_local_parent.y).max(min_height);

        window.boundary.dimensions = Vec2::new(new_width, new_height);

        window_transform.translation.x = match state.corner {
            ResizeCorner::BottomLeft => state.fixed_x - new_width * 0.5,
            ResizeCorner::BottomRight => state.fixed_x + new_width * 0.5,
        };
        window_transform.translation.y =
            state.fixed_top_y - window.header_height - new_height * 0.5;

        if let Some(root_entity) = window.root_entity {
            if let Ok(mut draggable) = draggables.get_mut(root_entity) {
                let edge_padding = 10.0;
                draggable.region = Some(DraggableRegion {
                    region: Vec2::new(new_width + edge_padding, window.header_height + edge_padding),
                    offset: Vec2::new(
                        window_transform.translation.x,
                        window_transform.translation.y + (new_height + window.header_height) * 0.5,
                    ),
                });
            }
        }
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

    /// Propagate any change on the Window to **all** its descendants.
    fn update(
        mut commands: Commands,
        mut windows:   Query<(Entity, &Window), Changed<Window>>,
        mut bodies:    Query<&mut WindowBody>,
        mut borders:   Query<&mut BorderedRectangle>,
        mut plus_sets: Query<(&mut Plus, &mut ColorChangeOn, &mut Clickable<WindowActions>)>,
        mut header_titles: Query<
            (Entity, &mut WindowTitle, &mut Text2d, &mut Transform),
            (Without<WindowHeader>, Without<WindowCloseButton>)
        >,
        children_q:    Query<&Children>,
        // ─────────── ParamSet holds the two conflicting queries ───────
        mut hb: ParamSet<(
            Query<(&mut WindowHeader, &mut Transform)>,
            Query<(&mut WindowCloseButton, &mut Transform)>,
        )>,
    ) {
        for (win_ent, win) in &mut windows {
            if let Ok(children) = children_q.get(win_ent) {
                for &child in children {
                    /* ---------- BODY ---------- */
                    if let Ok(mut body) = bodies.get_mut(child) {
                        body.boundary = win.boundary;
                        if let Ok(gkids) = children_q.get(child) {
                            for &g in gkids {
                                if let Ok(mut border) = borders.get_mut(g) {
                                    border.boundary = win.boundary;
                                }
                            }
                        }
                    }
    
                    /* ---------- HEADER ---------- */
                        if let Ok((mut header, mut tf)) = hb.p0().get_mut(child) {
                            header.boundary.dimensions =
                                Vec2::new(win.boundary.dimensions.x, win.header_height);
                            header.boundary.thickness = win.boundary.thickness;
                            header.title = win.title.clone();
                            tf.translation =
                                Vec3::new(0.0, (win.boundary.dimensions.y + win.header_height) / 2.0, 0.0);
    
                            let mut title_entity: Option<Entity> = None;
                            if let Ok(gkids) = children_q.get(child) {
                                for &g in gkids {
                                    if let Ok(mut border) = borders.get_mut(g) {
                                        border.boundary = header.boundary;
                                    }
                                    if let Ok((entity, mut title, mut text, mut title_tf)) =
                                        header_titles.get_mut(g)
                                    {
                                        title_entity = Some(entity);
                                        if let Some(new_title) = &win.title {
                                            *title = new_title.clone();
                                            text.0 = new_title.text.clone();
                                            title_tf.translation = Vec3::new(
                                                (-header.boundary.dimensions.x + new_title.padding) / 2.0,
                                                0.0,
                                                0.0,
                                            );
                                        }
                                    }
                                }
                            }

                            match (&win.title, title_entity) {
                                (Some(new_title), None) => {
                                    commands.entity(child).with_children(|parent| {
                                        parent.spawn((
                                            new_title.clone(),
                                            Text2d(new_title.text.clone()),
                                            TextColor(PRIMARY_COLOR),
                                            TextFont {
                                                font_size: 12.0,
                                                ..default()
                                            },
                                            Anchor::CENTER_LEFT,
                                            Transform::from_xyz(
                                                (-header.boundary.dimensions.x + new_title.padding) / 2.0,
                                                0.0,
                                                0.0,
                                            ),
                                        ));
                                    });
                                }
                                (None, Some(entity)) => {
                                    commands.entity(entity).despawn();
                                }
                                _ => {}
                            }
                        }
    
                    /* ---------- CLOSE BUTTON ---------- */
                    if let Ok((mut btn, mut tf)) = hb.p1().get_mut(child) {
                        btn.boundary.dimensions = Vec2::splat(win.header_height);
                        btn.boundary.thickness = win.boundary.thickness;
                        tf.translation = Vec3::new(
                            (win.boundary.dimensions.x - win.header_height) / 2.0,
                            (win.boundary.dimensions.y + win.header_height) / 2.0,
                            0.0,
                        );
    
                        if let Ok(gkids) = children_q.get(child) {
                            for &g in gkids {
                                if let Ok(mut border) = borders.get_mut(g) {
                                    border.boundary = btn.boundary;
                                }
                                if let Ok((mut plus, mut cc, mut click)) = plus_sets.get_mut(g) {
                                    plus.dimensions = btn.boundary.dimensions - 10.0;
                                    let region = Some(btn.boundary.dimensions + 10.0);
                                    click.region = region;
                                    for ev in &mut cc.events {
                                        if let ColorChangeEvent::Hover(_, r)
                                        | ColorChangeEvent::Click(_, r) = ev
                                        {
                                            *r = region;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext

    ) {
        let (title, boundary, header_h, close_btn, root_entity) = {
            let mut entity_mut = world.entity_mut(entity);
            let mut w = entity_mut.get_mut::<Window>().unwrap();
            // ensure root is self if not set
            if w.root_entity.is_none() {
                w.root_entity = Some(entity);
            }
            (
                w.title.clone(),
                w.boundary,
                w.header_height,
                w.has_close_button,
                w.root_entity
            )
        };

        /*  Spawn children *directly* under Window  */
        world.commands().entity(entity).with_children(|parent| {
            // Body -------------
            parent.spawn(WindowBody { boundary });

            // Header -----------
            let header_boundary = HollowRectangle {
                dimensions: Vec2::new(boundary.dimensions.x, header_h),
                ..boundary
            };
            parent.spawn((
                WindowHeader {
                    title,
                    boundary: header_boundary,
                    has_close_button: false, // header never spawns it now
                    root_entity,
                },
                Transform::from_xyz(
                    0.0,
                    (boundary.dimensions.y + header_h) / 2.0,
                    0.0,
                ),
            ));

            // Close button -----
            if close_btn {
                parent.spawn((
                    WindowCloseButton {
                        boundary: HollowRectangle {
                            dimensions: Vec2::splat(header_h),
                            thickness: boundary.thickness,
                            color: boundary.color,
                            ..default()
                        },
                        root_entity,
                    },
                    Transform::from_xyz(
                        (boundary.dimensions.x - header_h) / 2.0,
                        (boundary.dimensions.y + header_h) / 2.0,
                        0.0,
                    ),
                ));
            }
        });
    }
}

#[derive(Clone, Component)]
#[component(on_insert = WindowBody::on_insert)]
#[require(Transform, Visibility)]
pub struct WindowBody {
    pub boundary: HollowRectangle,
}

impl WindowBody {
    fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let boundary = world.entity(entity).get::<WindowBody>().unwrap().boundary;
        world
            .commands()
            .entity(entity)
            .with_children(|p| {p.spawn(BorderedRectangle { boundary });});
    }
}

/* ─────────────────────────  HEADER  ───────────────────────── */

#[derive(Clone, Component)]
#[component(on_insert = WindowHeader::on_insert)]
#[require(Transform, Visibility)]
pub struct WindowHeader {
    pub title: Option<WindowTitle>,
    pub boundary: HollowRectangle,
    pub has_close_button: bool,
    pub root_entity: Option<Entity>,
}

impl Default for WindowHeader {
    fn default() -> Self {
        Self {
            title: None,
            boundary: HollowRectangle::default(),
            has_close_button: false,
            root_entity: None,
        }
    }
}

impl WindowHeader {
    fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let (root_entity, boundary, has_btn, title, color) = {
            let h = world.entity(entity).get::<WindowHeader>().unwrap();
            (
                h.root_entity,
                h.boundary,
                h.has_close_button,
                h.title.clone(),
                h.boundary.color,
            )
        };

        world.commands().entity(entity).with_children(|parent| {
            parent.spawn(BorderedRectangle { boundary });

            if has_btn {
                parent.spawn((
                    WindowCloseButton {
                        boundary: HollowRectangle {
                            dimensions: Vec2::splat(boundary.dimensions.y),
                            thickness: boundary.thickness,
                            color,
                            ..default()
                        },
                        root_entity,
                    },
                    Transform::from_xyz(
                        (boundary.dimensions.x - boundary.dimensions.y) / 2.0,
                        0.0,
                        0.0,
                    ),
                ));
            }

            if let Some(t) = title {
                parent.spawn((
                    t.clone(),
                    Text2d(t.text),
                    TextColor(PRIMARY_COLOR),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    Anchor::CENTER_LEFT,
                    Transform::from_xyz(
                        (-boundary.dimensions.x + t.padding) / 2.0,
                        0.0,
                        0.0,
                    ),
                ));
            }
        });
    }
}

#[derive(Clone, Component)]
#[component(on_insert = WindowCloseButton::on_insert)]
#[require(Transform, Visibility)]
pub struct WindowCloseButton {
    pub boundary: HollowRectangle,
    pub root_entity: Option<Entity>,
}

impl WindowCloseButton {
    fn on_insert(
        mut world: DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let (root, boundary, dimensions, color) = {
            let b = world.entity(entity).get::<WindowCloseButton>().unwrap();
            (
                b.root_entity,
                b.boundary,
                b.boundary.dimensions,
                b.boundary.color,
            )
        };

        let handle =
            world.get_resource::<AssetServer>().unwrap().load("./audio/effects/mouse_click.ogg");

        world.commands().entity(entity).with_children(|parent| {
            parent.spawn((
                WindowCloseButtonBorder,
                BorderedRectangle { boundary },
            ));

            let interaction_region = dimensions + 10.0;
            let icon_dim = dimensions - 10.0;

            parent.spawn((
                WindowCloseButtonIcon,
                Plus {
                    dimensions: icon_dim,
                    thickness: 1.0,
                    color,
                },
                Clickable::with_region(
                    vec![WindowActions::CloseWindow],
                    interaction_region,
                ),
                ColorChangeOn::new(vec![
                    ColorChangeEvent::Click(vec![CLICKED_BUTTON], Some(interaction_region)),
                    ColorChangeEvent::Hover(vec![HOVERED_BUTTON], Some(interaction_region)),
                ]),
                ColorAnchor(color),
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
