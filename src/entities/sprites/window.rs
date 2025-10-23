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
        interaction::{ActionPallet, Clickable, InputAction},
    },
};

/* ─────────────────────────  PLUGIN  ───────────────────────── */

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Window::update);
    }
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

    /// Propagate any change on the Window to **all** its descendants.
    fn update(
        mut windows:   Query<(Entity, &Window), Changed<Window>>,
        mut bodies:    Query<&mut WindowBody>,
        mut borders:   Query<&mut BorderedRectangle>,
        mut plus_sets: Query<(&mut Plus, &mut ColorChangeOn, &mut Clickable<WindowActions>)>,
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
    
                        if let Ok(gkids) = children_q.get(child) {
                            for &g in gkids {
                                if let Ok(mut border) = borders.get_mut(g) {
                                    border.boundary = header.boundary;
                                }
                            }
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