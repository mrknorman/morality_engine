//! Reusable delayed hover help primitive.
//!
//! Hover boxes are owner-scoped, layer-scoped overlays that appear after a
//! configurable hover delay and clamp to an owner-local region.
use std::collections::HashMap;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
    text::TextBounds,
};

use crate::{
    data::states::PauseState,
    entities::sprites::compound::HollowRectangle,
    startup::cursor::CustomCursor,
    systems::{
        interaction::{
            interaction_gate_allows_for_owner, Hoverable, InteractionCapture, InteractionCaptureOwner,
            InteractionGate,
        },
        ui::layer::{self, UiLayer, UiLayerKind},
    },
};

const EPSILON: f32 = 0.0001;

#[derive(Component, Clone, Copy, Debug)]
#[require(
    HoverBoxState,
    HoverBoxDelay,
    HoverBoxStyle,
    Sprite,
    Transform,
    Visibility
)]
#[component(on_insert = HoverBoxRoot::on_insert)]
pub struct HoverBoxRoot {
    pub owner: Entity,
    pub input_layer: UiLayerKind,
    pub clamp_size: Vec2,
}

impl HoverBoxRoot {
    pub const fn new(owner: Entity, input_layer: UiLayerKind, clamp_size: Vec2) -> Self {
        Self {
            owner,
            input_layer,
            clamp_size,
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let Some(root) = world.entity(entity).get::<HoverBoxRoot>().copied() else {
            return;
        };
        let style = world
            .entity(entity)
            .get::<HoverBoxStyle>()
            .copied()
            .unwrap_or_default();

        if world.entity(entity).get::<UiLayer>().is_none() {
            world
                .commands()
                .entity(entity)
                .insert(UiLayer::new(root.owner, root.input_layer));
        }

        let mut transform = world
            .entity(entity)
            .get::<Transform>()
            .copied()
            .unwrap_or_default();
        transform.translation.z = style.z;
        world.commands().entity(entity).insert((
            transform,
            Sprite::from_color(style.background_color, style.size),
            Visibility::Hidden,
        ));

        let mut has_border = false;
        let mut has_label = false;
        if let Some(children) = world.entity(entity).get::<Children>() {
            for child in children.iter() {
                if world.entity(child).contains::<HoverBoxBorder>() {
                    has_border = true;
                }
                if world.entity(child).contains::<HoverBoxLabel>() {
                    has_label = true;
                }
            }
        }
        if has_border && has_label {
            return;
        }

        world.commands().entity(entity).with_children(|tooltip| {
            if !has_border {
                tooltip.spawn((
                    Name::new("hover_box_border"),
                    HoverBoxBorder,
                    HollowRectangle {
                        dimensions: Vec2::new(
                            (style.size.x - 2.0).max(1.0),
                            (style.size.y - 2.0).max(1.0),
                        ),
                        thickness: style.border_thickness,
                        color: style.border_color,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.01),
                ));
            }
            if !has_label {
                tooltip.spawn((
                    Name::new("hover_box_label"),
                    HoverBoxLabel,
                    Text2d::new(String::new()),
                    TextFont {
                        font_size: style.font_size,
                        ..default()
                    },
                    TextColor(style.text_color),
                    TextBounds {
                        width: Some((style.size.x - style.text_padding.x * 2.0).max(1.0)),
                        height: Some((style.size.y - style.text_padding.y * 2.0).max(1.0)),
                    },
                    TextLayout {
                        justify: Justify::Left,
                        ..default()
                    },
                    Anchor::CENTER_LEFT,
                    Transform::from_xyz(-style.size.x * 0.5 + style.text_padding.x, 0.0, 0.02),
                ));
            }
        });
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct HoverBoxDelay(pub f32);

impl Default for HoverBoxDelay {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct HoverBoxTarget {
    pub root: Entity,
    pub anchor_offset: Vec2,
    pub target_size: Vec2,
    pub hover_region: Option<Vec2>,
    pub hover_region_offset: Vec2,
}

impl HoverBoxTarget {
    pub const fn new(root: Entity, target_size: Vec2) -> Self {
        Self {
            root,
            anchor_offset: Vec2::ZERO,
            target_size,
            hover_region: None,
            hover_region_offset: Vec2::ZERO,
        }
    }

    pub const fn with_hover_region(mut self, region: Vec2, offset: Vec2) -> Self {
        self.hover_region = Some(region);
        self.hover_region_offset = offset;
        self
    }
}

#[derive(Component, Clone, Debug, Default)]
pub struct HoverBoxContent {
    pub text: String,
}

impl HoverBoxContent {
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct HoverBoxStyle {
    pub size: Vec2,
    pub text_padding: Vec2,
    pub target_gap: f32,
    pub clamp_margin: Vec2,
    pub border_thickness: f32,
    pub background_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub font_size: f32,
    pub z: f32,
}

impl Default for HoverBoxStyle {
    fn default() -> Self {
        Self {
            size: Vec2::new(340.0, 72.0),
            text_padding: Vec2::new(14.0, 8.0),
            target_gap: 10.0,
            clamp_margin: Vec2::splat(14.0),
            border_thickness: 2.0,
            background_color: Color::BLACK,
            border_color: Color::srgb(0.10, 2.60, 0.25),
            text_color: Color::srgb(0.10, 2.60, 0.25),
            font_size: 14.0,
            z: 1.35,
        }
    }
}

impl HoverBoxStyle {
    pub const fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub const fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub const fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
struct HoverBoxState {
    active_target: Option<Entity>,
    hovered_seconds: f32,
}

#[derive(Component)]
pub struct HoverBoxLabel;

#[derive(Component)]
pub struct HoverBoxBorder;

#[derive(Clone, Debug)]
struct HoverBoxCandidate {
    target_entity: Entity,
    target_center_world: Vec2,
    target_size: Vec2,
    anchor_offset: Vec2,
    content: String,
    z: f32,
    rank: u64,
}

pub fn spawn_hover_box_root(
    parent: &mut ChildSpawnerCommands<'_>,
    name: impl Into<String>,
    owner: Entity,
    input_layer: UiLayerKind,
    clamp_size: Vec2,
    style: HoverBoxStyle,
    delay_seconds: f32,
) -> Entity {
    let name: String = name.into();
    parent
        .spawn((
            Name::new(name),
            HoverBoxDelay(delay_seconds),
            style,
            HoverBoxRoot::new(owner, input_layer, clamp_size),
        ))
        .id()
}

fn cursor_within_region(
    cursor_position: Vec2,
    global_transform: &GlobalTransform,
    region: Vec2,
    offset: Vec2,
) -> bool {
    let half = region * 0.5;
    let inverse = global_transform.to_matrix().inverse();
    let local = inverse.transform_point3(cursor_position.extend(0.0)).truncate();
    local.x >= offset.x - half.x
        && local.x <= offset.x + half.x
        && local.y >= offset.y - half.y
        && local.y <= offset.y + half.y
}

fn reduce_hover_box_visibility(
    state: &mut HoverBoxState,
    candidate_target: Option<Entity>,
    delay_seconds: f32,
    delta_seconds: f32,
) -> bool {
    let Some(candidate_target) = candidate_target else {
        state.active_target = None;
        state.hovered_seconds = 0.0;
        return false;
    };

    if state.active_target != Some(candidate_target) {
        state.active_target = Some(candidate_target);
        state.hovered_seconds = 0.0;
        return false;
    }

    state.hovered_seconds += delta_seconds.max(0.0);
    state.hovered_seconds + EPSILON >= delay_seconds.max(0.0)
}

#[inline]
fn reset_hover_box_state(state: &mut HoverBoxState) {
    state.active_target = None;
    state.hovered_seconds = 0.0;
}

#[inline]
fn hide_hover_box(visibility: &mut Visibility) {
    *visibility = Visibility::Hidden;
}

#[inline]
fn is_preferred_candidate(candidate: &HoverBoxCandidate, current: &HoverBoxCandidate) -> bool {
    candidate.z > current.z
        || ((candidate.z - current.z).abs() <= EPSILON && candidate.rank > current.rank)
}

fn sync_hover_boxes(
    time: Res<Time<Real>>,
    cursor: Res<CustomCursor>,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    owner_global_query: Query<&GlobalTransform>,
    mut root_queries: ParamSet<(
        Query<(Entity, &UiLayer, Option<&Visibility>, Option<&InteractionGate>)>,
        Query<
            (
                Entity,
                &HoverBoxRoot,
                &HoverBoxStyle,
                &HoverBoxDelay,
            ),
        >,
        Query<
            (
                Entity,
                &HoverBoxRoot,
                &HoverBoxStyle,
                &HoverBoxDelay,
                &mut HoverBoxState,
                &mut Transform,
                &mut Sprite,
                &mut Visibility,
            ),
            (With<HoverBoxRoot>, Without<HoverBoxLabel>),
        >,
    )>,
    target_query: Query<
        (
            Entity,
            &HoverBoxTarget,
            &HoverBoxContent,
            &Hoverable,
            &GlobalTransform,
            Option<&InheritedVisibility>,
            Option<&InteractionGate>,
        ),
    >,
    mut label_query: Query<
        (
            &ChildOf,
            &mut Text2d,
            &mut TextFont,
            &mut TextColor,
            &mut TextBounds,
            &mut Transform,
        ),
        With<HoverBoxLabel>,
    >,
    mut border_query: Query<(&ChildOf, &mut HollowRectangle), With<HoverBoxBorder>>,
) {
    // Query-safety contract:
    // - root layer reads (`p0`) are disjoint from root mutation (`p2`) via ParamSet
    // - labels/borders are child marker queries and only mutate their own components
    // - target query is read-only hover/content input
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &root_queries.p0());

    let mut root_meta = HashMap::new();
    {
        let root_query = root_queries.p1();
        for (entity, root, style, delay) in root_query.iter() {
            root_meta.insert(entity, (*root, *style, *delay));
        }
    }

    let mut candidate_by_root: HashMap<Entity, HoverBoxCandidate> = HashMap::new();
    if let Some(cursor_position) = cursor.position {
        for (
            target_entity,
            target,
            content,
            hoverable,
            target_global,
            inherited_visibility,
            gate,
        ) in target_query.iter()
        {
            if !hoverable.hovered || content.is_empty() {
                continue;
            }
            if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
                continue;
            }
            let Some((root, _, _)) = root_meta.get(&target.root).copied() else {
                continue;
            };
            if !interaction_gate_allows_for_owner(gate, pause_state.as_ref(), &capture_query, root.owner)
            {
                continue;
            }
            if layer::active_layer_kind_for_owner(&active_layers, root.owner) != root.input_layer {
                continue;
            }
            if let Some(region) = target.hover_region {
                if !cursor_within_region(cursor_position, target_global, region, target.hover_region_offset)
                {
                    continue;
                }
            }

            let candidate = HoverBoxCandidate {
                target_entity,
                target_center_world: target_global.translation().truncate(),
                target_size: target.target_size,
                anchor_offset: target.anchor_offset,
                content: content.text.clone(),
                z: target_global.translation().z,
                rank: target_entity.to_bits(),
            };
            match candidate_by_root.get_mut(&target.root) {
                Some(current) if is_preferred_candidate(&candidate, current) => {
                    *current = candidate;
                }
                None => {
                    candidate_by_root.insert(target.root, candidate);
                }
                _ => {}
            }
        }
    }

    let delta_seconds = time.delta_secs();
    for (
        root_entity,
        root,
        style,
        delay,
        mut state,
        mut root_transform,
        mut sprite,
        mut visibility,
    ) in root_queries.p2().iter_mut()
    {
        if layer::active_layer_kind_for_owner(&active_layers, root.owner) != root.input_layer {
            reset_hover_box_state(&mut state);
            hide_hover_box(&mut visibility);
            continue;
        }

        let candidate = candidate_by_root.get(&root_entity);
        let should_show = reduce_hover_box_visibility(
            &mut state,
            candidate.map(|candidate| candidate.target_entity),
            delay.0,
            delta_seconds,
        );
        if !should_show {
            hide_hover_box(&mut visibility);
            continue;
        }

        let Some(candidate) = candidate else {
            hide_hover_box(&mut visibility);
            continue;
        };

        let Ok(owner_global) = owner_global_query.get(root.owner) else {
            hide_hover_box(&mut visibility);
            continue;
        };

        sprite.custom_size = Some(style.size);
        sprite.color = style.background_color;

        let mut hover_center_world = candidate.target_center_world + candidate.anchor_offset;
        let target_half_height = candidate.target_size.y * 0.5;
        let box_half_size = style.size * 0.5;
        hover_center_world.y -= target_half_height + style.target_gap + box_half_size.y;

        let owner_center_world = owner_global.translation().truncate();
        let half_clamp = root.clamp_size * 0.5;
        if half_clamp.x > 0.0 {
            let min_x = owner_center_world.x - half_clamp.x + box_half_size.x + style.clamp_margin.x;
            let max_x = owner_center_world.x + half_clamp.x - box_half_size.x - style.clamp_margin.x;
            if min_x <= max_x {
                hover_center_world.x = hover_center_world.x.clamp(min_x, max_x);
            }
        }
        if half_clamp.y > 0.0 {
            let min_y = owner_center_world.y - half_clamp.y + box_half_size.y + style.clamp_margin.y;
            let max_y = owner_center_world.y + half_clamp.y - box_half_size.y - style.clamp_margin.y;
            if min_y <= max_y {
                hover_center_world.y = hover_center_world.y.clamp(min_y, max_y);
            }
        }

        let owner_world_to_local = owner_global.to_matrix().inverse();
        let local_center = owner_world_to_local.transform_point3(hover_center_world.extend(0.0));
        root_transform.translation.x = local_center.x;
        root_transform.translation.y = local_center.y;
        root_transform.translation.z = style.z;

        for (parent, mut border) in border_query.iter_mut() {
            if parent.parent() != root_entity {
                continue;
            }
            border.dimensions = Vec2::new((style.size.x - 2.0).max(1.0), (style.size.y - 2.0).max(1.0));
            border.thickness = style.border_thickness;
            border.color = style.border_color;
            break;
        }

        for (parent, mut text, mut font, mut text_color, mut bounds, mut label_transform) in
            label_query.iter_mut()
        {
            if parent.parent() != root_entity {
                continue;
            }
            if text.0 != candidate.content {
                text.0 = candidate.content.clone();
            }
            font.font_size = style.font_size;
            text_color.0 = style.text_color;
            bounds.width = Some((style.size.x - style.text_padding.x * 2.0).max(1.0));
            bounds.height = Some((style.size.y - style.text_padding.y * 2.0).max(1.0));
            label_transform.translation.x = -style.size.x * 0.5 + style.text_padding.x;
            label_transform.translation.y = 0.0;
            label_transform.translation.z = 0.02;
            break;
        }

        *visibility = Visibility::Visible;
    }
}

pub struct HoverBoxPlugin;

impl Plugin for HoverBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_hover_boxes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ui::layer::UiLayer;

    #[test]
    fn hover_reducer_respects_delay_and_target_switch() {
        let target_a = Entity::from_bits(1);
        let target_b = Entity::from_bits(2);
        let mut state = HoverBoxState::default();

        assert!(!reduce_hover_box_visibility(&mut state, Some(target_a), 0.5, 0.25));
        assert!(!reduce_hover_box_visibility(&mut state, Some(target_a), 0.5, 0.24));
        assert!(!reduce_hover_box_visibility(&mut state, Some(target_a), 0.5, 0.25));
        assert!(reduce_hover_box_visibility(&mut state, Some(target_a), 0.5, 0.01));

        assert!(!reduce_hover_box_visibility(&mut state, Some(target_b), 0.5, 0.5));
        assert!(reduce_hover_box_visibility(&mut state, Some(target_b), 0.5, 0.5));

        assert!(!reduce_hover_box_visibility(&mut state, None, 0.5, 0.5));
        assert_eq!(state.active_target, None);
        assert!(state.hovered_seconds <= EPSILON);
    }

    #[test]
    fn root_layer_gating_hides_until_matching_layer_becomes_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, sync_hover_boxes);

        let mut cursor = CustomCursor::default();
        cursor.position = Some(Vec2::ZERO);
        app.insert_resource(cursor);
        app.init_resource::<Time<Real>>();

        let owner = app
            .world_mut()
            .spawn((Transform::default(), GlobalTransform::default()))
            .id();
        app.world_mut().spawn((
            UiLayer::new(owner, UiLayerKind::Base),
            Visibility::Visible,
        ));
        let dropdown_layer = app
            .world_mut()
            .spawn((
                UiLayer::new(owner, UiLayerKind::Dropdown),
                Visibility::Hidden,
            ))
            .id();

        let root = app
            .world_mut()
            .spawn((
                HoverBoxRoot::new(owner, UiLayerKind::Dropdown, Vec2::new(300.0, 300.0)),
                HoverBoxDelay(0.0),
                HoverBoxStyle::default(),
                HoverBoxState::default(),
                Transform::default(),
                GlobalTransform::default(),
                Sprite::from_color(Color::BLACK, Vec2::new(100.0, 40.0)),
                Visibility::Hidden,
            ))
            .id();
        app.world_mut().spawn((
            HoverBoxTarget::new(root, Vec2::new(100.0, 24.0)),
            HoverBoxContent {
                text: "tooltip".to_string(),
            },
            Hoverable { hovered: true },
            Transform::default(),
            GlobalTransform::default(),
        ));

        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(root),
            Some(&Visibility::Hidden)
        );

        *app.world_mut()
            .get_mut::<Visibility>(dropdown_layer)
            .expect("dropdown layer visibility") = Visibility::Visible;
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_millis(16));
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(root),
            Some(&Visibility::Visible)
        );
    }

    #[test]
    fn hover_box_root_insertion_adds_required_components_and_children() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();
        let root = world.spawn(HoverBoxRoot::new(owner, UiLayerKind::Base, Vec2::splat(240.0))).id();

        assert!(world.entity(root).get::<HoverBoxState>().is_some());
        assert!(world.entity(root).get::<HoverBoxDelay>().is_some());
        assert!(world.entity(root).get::<HoverBoxStyle>().is_some());
        assert!(world.entity(root).get::<Sprite>().is_some());
        assert!(world.entity(root).get::<Transform>().is_some());
        assert!(world.entity(root).get::<Visibility>().is_some());
        assert_eq!(
            world.entity(root).get::<UiLayer>(),
            Some(&UiLayer::new(owner, UiLayerKind::Base))
        );

        let children = world.entity(root).get::<Children>().expect("hover box children");
        let mut has_label = false;
        let mut has_border = false;
        for child in children.iter() {
            if world.entity(child).contains::<HoverBoxLabel>() {
                has_label = true;
            }
            if world.entity(child).contains::<HoverBoxBorder>() {
                has_border = true;
            }
        }
        assert!(has_label);
        assert!(has_border);
    }
}
