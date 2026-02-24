use std::{collections::HashMap, f32::consts::FRAC_PI_4};

// Migration checklist (`docs/ui_architecture_contract.md`):
// - migrate window interaction gating to unified input policy/state
// - keep drag/close/resize focus ownership deterministic per root window
// - remove legacy gate propagation once unified resolver is live

use bevy::{
    camera::primitives::Aabb,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
    window::PrimaryWindow,
};
use enum_map::{enum_map, Enum};

use crate::{
    entities::sprites::compound::*,
    entities::text::{scaled_font_size, Cell, Column, Row, Table, TextContent},
    startup::{cursor::CustomCursor, render::OffscreenCamera},
    systems::{
        audio::{TransientAudio, TransientAudioPallet},
        colors::{ColorAnchor, CLICKED_BUTTON, HOVERED_BUTTON, PRIMARY_COLOR},
        interaction::{
            scoped_owner_has_focus, ui_input_policy_allows_mode, ActionPallet, Clickable,
            Draggable, DraggableRegion, DraggableViewportBounds, Hoverable, InputAction,
            InteractionSystem, InteractionVisualPalette, InteractionVisualState,
            SelectableClickActivation, SelectableMenu, SelectableScopeOwner, UiInputPolicy,
            UiInteractionState,
        },
        ui::scroll::{
            ScrollAxis, ScrollBackend, ScrollBar, ScrollState, ScrollZoomConfig, ScrollableContent,
            ScrollableContentExtent, ScrollableRoot, ScrollableViewport,
        },
        ui::{
            layer::UiLayerKind,
            selector::SelectorSurface,
            tabs::{TabBar, TabBarState, TabItem},
        },
    },
};

/* ─────────────────────────  PLUGIN  ───────────────────────── */

pub struct WindowPlugin;
impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowZStack>()
            .init_resource::<ActiveWindowInteraction>()
            .init_resource::<UiInteractionState>()
            .configure_sets(
                Update,
                (
                    WindowSystem::Input,
                    WindowSystem::Resolve.after(WindowSystem::Input),
                    WindowSystem::Layout.after(WindowSystem::Resolve),
                )
                    .before(InteractionSystem::Hoverable),
            )
            .add_systems(
                Update,
                (
                    Window::raise_window_on_pointer_down,
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
                    Window::sync_tab_row_layout,
                    Window::route_explicit_window_content,
                    Window::sync_scroll_runtime_geometry,
                    Window::sync_horizontal_proxy_offset,
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
            )
            .add_systems(
                Update,
                Window::sync_tab_row_visuals
                    .after(InteractionSystem::Selectable)
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
const WINDOW_FOCUS_DEPTH_SPAN: f32 = 60.0;
const WINDOW_RESIZE_HANDLE_SIZE: f32 = 20.0;
const WINDOW_MIN_WIDTH: f32 = 60.0;
const WINDOW_MIN_HEIGHT: f32 = 40.0;
const WINDOW_CLOSE_BUTTON_MIN_CLICK_REGION: f32 = 8.0;
const WINDOW_CLOSE_BUTTON_CLICK_REGION_SCALE: f32 = 1.1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ResizeCorner {
    BottomLeft,
    BottomRight,
}

fn close_button_click_region(close_button_side: f32) -> Vec2 {
    Vec2::splat(
        (close_button_side * WINDOW_CLOSE_BUTTON_CLICK_REGION_SCALE)
            .max(WINDOW_CLOSE_BUTTON_MIN_CLICK_REGION),
    )
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

#[derive(Component, Clone)]
#[require(Window)]
#[component(on_insert = WindowTabRow::on_insert)]
pub struct WindowTabRow {
    pub labels: Vec<String>,
    pub tab_width: f32,
    pub row_height: f32,
    pub text_size: f32,
    pub selected_text_size: f32,
    pub color: Color,
    pub z: f32,
    pub click_activation: SelectableClickActivation,
}

impl Default for WindowTabRow {
    fn default() -> Self {
        Self {
            labels: vec![],
            tab_width: 120.0,
            row_height: 40.0,
            text_size: scaled_font_size(14.0),
            selected_text_size: scaled_font_size(21.0),
            color: Color::WHITE,
            z: 0.2,
            click_activation: SelectableClickActivation::HoveredOnly,
        }
    }
}

impl WindowTabRow {
    pub fn from_labels(labels: &[&str]) -> Self {
        Self {
            labels: labels.iter().map(|label| (*label).to_string()).collect(),
            ..default()
        }
    }

    pub fn with_tab_width(mut self, tab_width: f32) -> Self {
        self.tab_width = tab_width;
        self
    }

    pub fn with_row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    fn total_width(&self) -> f32 {
        self.tab_width.max(1.0) * self.labels.len().max(1) as f32
    }

    fn build_table(&self) -> Table {
        let tab_count = self.labels.len().max(1);
        let tab_column_width = self.total_width() / tab_count as f32;
        let tab_border_sides = RectangleSides {
            top: true,
            bottom: true,
            left: true,
            right: true,
        };
        let columns = self
            .labels
            .iter()
            .map(|label| {
                Column::new(
                    vec![Cell::new(TextContent::new(
                        label.clone(),
                        self.color,
                        self.text_size,
                    ))],
                    tab_column_width,
                    Vec2::new(8.0, 6.0),
                    Anchor::CENTER,
                    false,
                )
                .with_cell_boundary_sides(tab_border_sides)
                .with_cell_boundary_color(self.color)
            })
            .collect();
        Table {
            columns,
            rows: vec![Row {
                height: self.row_height.max(1.0),
            }],
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (tab_row, owner_entity) = {
            let Some(tab_row) = world.entity(entity).get::<WindowTabRow>().cloned() else {
                return;
            };
            let owner_entity = world
                .entity(entity)
                .get::<Window>()
                .and_then(|window| window.root_entity)
                .unwrap_or(entity);
            (tab_row, owner_entity)
        };
        if tab_row.labels.is_empty() {
            return;
        }

        let total_width = tab_row.total_width();
        let row_height = tab_row.row_height.max(1.0);
        let tab_width = tab_row.tab_width.max(1.0);
        let tab_hitbox = Vec2::new((tab_width - 8.0).max(8.0), (row_height - 4.0).max(8.0));
        let table_z = tab_row.z;
        let interaction_z = table_z + 0.02;
        let click_activation = tab_row.click_activation;

        let mut tab_root = Entity::PLACEHOLDER;
        let mut table_entity = Entity::PLACEHOLDER;
        world.commands().entity(entity).with_children(|parent| {
            tab_root = parent
                .spawn((
                    Name::new("window_tab_row_root"),
                    WindowTabRowInteractionRoot,
                    SelectableScopeOwner::new(owner_entity),
                    SelectableMenu::new(
                        0,
                        vec![KeyCode::ArrowLeft],
                        vec![KeyCode::ArrowRight, KeyCode::Tab],
                        vec![KeyCode::Enter],
                        true,
                    )
                    .with_click_activation(click_activation),
                    Transform::from_xyz(0.0, 0.0, interaction_z),
                ))
                .id();
            parent
                .commands()
                .entity(tab_root)
                .insert(TabBar::new(tab_root));

            table_entity = parent
                .spawn((
                    Name::new("window_tab_row_table"),
                    WindowTabRowTable { tab_root },
                    tab_row.build_table(),
                    Transform::from_xyz(-total_width * 0.5, 0.0, table_z),
                ))
                .id();

            for (index, _) in tab_row.labels.iter().enumerate() {
                let center_x = -total_width * 0.5 + tab_width * (index as f32 + 0.5);
                parent.spawn((
                    Name::new(format!("window_tab_row_target_{index}")),
                    WindowTabRowTarget { tab_root, index },
                    TabItem { index },
                    SelectorSurface::new(tab_root, index),
                    Clickable::<WindowActions>::with_region(vec![], tab_hitbox),
                    Transform::from_xyz(center_x, 0.0, interaction_z),
                ));
            }
        });

        world.commands().entity(entity).insert(WindowTabRowRuntime {
            tab_root,
            table_entity,
            total_width,
            tab_width,
            row_height,
            text_size: tab_row.text_size,
            selected_text_size: tab_row.selected_text_size,
            color: tab_row.color,
            z: tab_row.z,
        });
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct WindowTabRowTable {
    tab_root: Entity,
}

#[derive(Component, Clone, Copy, Debug)]
struct WindowTabRowInteractionRoot;

#[derive(Component, Clone, Copy, Debug)]
struct WindowTabRowTarget {
    tab_root: Entity,
    index: usize,
}

#[derive(Component, Clone, Debug)]
struct WindowTabRowRuntime {
    tab_root: Entity,
    table_entity: Entity,
    total_width: f32,
    tab_width: f32,
    row_height: f32,
    text_size: f32,
    selected_text_size: f32,
    color: Color,
    z: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct WindowContentHost {
    pub window_entity: Entity,
}

/// Explicit ownership marker for content that belongs inside a `UiWindow`'s
/// scrollable content slot.
///
/// Attach this to a content-root entity and parent your window content under
/// that root. This avoids brittle implicit "all non-chrome children" routing.
#[derive(Component, Clone, Copy, Debug)]
pub struct WindowContent {
    pub window_entity: Entity,
}

impl WindowContent {
    pub const fn new(window_entity: Entity) -> Self {
        Self { window_entity }
    }
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

#[derive(Component, Clone, Copy, Debug)]
struct WindowScrollRuntime {
    vertical_root: Entity,
    horizontal_root: Entity,
    horizontal_proxy: Entity,
    content_root: Entity,
    vertical_bar: Entity,
    horizontal_bar: Entity,
}

#[derive(Component, Clone, Copy, Debug)]
struct WindowScrollManaged;

#[derive(Component, Clone, Copy, Debug)]
struct WindowScrollHorizontalProxy;

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
    fn header_drag_region(
        window_width: f32,
        header_height: f32,
        has_close_button: bool,
    ) -> (Vec2, f32) {
        let vertical_padding = 10.0;
        if !has_close_button {
            return (
                Vec2::new(
                    window_width + vertical_padding,
                    header_height + vertical_padding,
                ),
                0.0,
            );
        }

        // Reserve exactly the close hitbox span on the right side of the
        // header (plus a tiny safety buffer), so there is no dead zone
        // between drag and close interactions.
        let close_region = close_button_click_region(header_height);
        let reserved_right = 0.5 * header_height + 0.5 * close_region.x + 0.5;
        let width = (window_width - reserved_right).max(24.0);
        (
            Vec2::new(width, header_height + vertical_padding),
            -reserved_right * 0.5,
        )
    }

    fn raise_window_on_pointer_down(
        mouse_input: Res<ButtonInput<MouseButton>>,
        cursor: Res<CustomCursor>,
        windows: Query<(Entity, &Window, &GlobalTransform)>,
        q_window_base: Query<&WindowBaseZ>,
        mut root_transforms: Query<&mut Transform>,
    ) {
        #[derive(Clone, Copy)]
        struct RootEntry {
            root_entity: Entity,
            current_z: f32,
            base_z: f32,
        }

        if !mouse_input.just_pressed(MouseButton::Left) {
            return;
        }
        let Some(cursor_position) = cursor.position else {
            return;
        };

        let mut top_candidate: Option<(Entity, f32)> = None;
        let mut entries_by_root: HashMap<Entity, RootEntry> = HashMap::new();
        for (window_entity, window, global_transform) in windows.iter() {
            let z = global_transform.translation().z;
            let root_entity = window.root_entity.unwrap_or(window_entity);
            let base_z = q_window_base.get(root_entity).map_or(z, |base| base.0);

            entries_by_root
                .entry(root_entity)
                .and_modify(|entry| {
                    if z > entry.current_z {
                        entry.current_z = z;
                    }
                })
                .or_insert(RootEntry {
                    root_entity,
                    current_z: z,
                    base_z,
                });

            if !Self::is_cursor_over_window_surface(cursor_position, window, global_transform) {
                continue;
            }

            let replace = match top_candidate {
                None => true,
                Some((current_entity, current_z)) => {
                    z > current_z
                        || (z == current_z && root_entity.index() > current_entity.index())
                }
            };
            if replace {
                top_candidate = Some((root_entity, z));
            }
        }

        let Some((focused_root, _)) = top_candidate else {
            return;
        };

        let mut ordered_entries: Vec<RootEntry> = entries_by_root.into_values().collect();
        if ordered_entries.len() <= 1 {
            return;
        }

        ordered_entries.sort_by(|a, b| {
            a.current_z
                .partial_cmp(&b.current_z)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.root_entity.index().cmp(&b.root_entity.index()))
        });

        let focused_position = ordered_entries
            .iter()
            .position(|entry| entry.root_entity == focused_root);
        let Some(focused_position) = focused_position else {
            return;
        };
        if focused_position + 1 != ordered_entries.len() {
            let focused_entry = ordered_entries.remove(focused_position);
            ordered_entries.push(focused_entry);
        }

        let base_anchor = ordered_entries
            .iter()
            .map(|entry| entry.base_z)
            .fold(f32::INFINITY, f32::min);
        let lowest_current = ordered_entries
            .iter()
            .map(|entry| entry.current_z)
            .fold(f32::INFINITY, f32::min);
        // Normalize around the currently lowest active window while keeping a
        // floor at the baseline stack anchor to prevent upward drift.
        let anchor = lowest_current.min(base_anchor);
        let step = if ordered_entries.len() > 1 {
            WINDOW_FOCUS_DEPTH_SPAN / (ordered_entries.len() - 1) as f32
        } else {
            0.0
        };

        for (index, entry) in ordered_entries.into_iter().enumerate() {
            if let Ok(mut root_transform) = root_transforms.get_mut(entry.root_entity) {
                root_transform.translation.z = anchor + index as f32 * step;
            }
        }
    }

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
        q_all_windows: Query<(Entity, &Window), With<Window>>,
        q_window_base: Query<&WindowBaseZ>,
        mut q_transforms: Query<&mut Transform>,
    ) {
        let mut added_roots: Vec<Entity> = q_added_windows
            .iter()
            .map(|(window_entity, window)| window.root_entity.unwrap_or(window_entity))
            .collect();
        added_roots.sort_by_key(|entity| entity.index());
        added_roots.dedup();
        if added_roots.is_empty() {
            return;
        }

        let mut all_roots: Vec<Entity> = q_all_windows
            .iter()
            .map(|(window_entity, window)| window.root_entity.unwrap_or(window_entity))
            .collect();
        all_roots.sort_by_key(|entity| entity.index());
        all_roots.dedup();

        let existing_root_count = all_roots.len().saturating_sub(added_roots.len());
        if existing_root_count == 0 {
            // Fresh stack after all windows were closed: restart depth ordering
            // from baseline so z does not drift upward across reopen cycles.
            z_stack.next_order = 0;
        }

        for root_entity in added_roots {
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
        interaction_state: Res<UiInteractionState>,
        mut active_interaction: ResMut<ActiveWindowInteraction>,
        parent_globals: Query<&GlobalTransform>,
        mut draggables: Query<&mut Draggable>,
        mut windows: ParamSet<(
            Query<(
                Entity,
                &Window,
                &GlobalTransform,
                Option<&ChildOf>,
                Option<&UiInputPolicy>,
            )>,
            Query<(
                Entity,
                &mut Window,
                &mut Transform,
                &GlobalTransform,
                Option<&ChildOf>,
                Option<&UiInputPolicy>,
                Option<&WindowContentMetrics>,
                Option<&WindowOverflowPolicy>,
                Option<&WindowTabRowRuntime>,
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
                for (entity, window, global_transform, _, gate) in windows.p0().iter() {
                    if !Self::window_interaction_allowed(
                        entity,
                        window,
                        gate,
                        &interaction_state,
                    ) {
                        continue;
                    }
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
                for (entity, window, global_transform, _, gate) in windows.p0().iter() {
                    if !Self::window_interaction_allowed(
                        entity,
                        window,
                        gate,
                        &interaction_state,
                    ) {
                        continue;
                    }
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
        let mut drag_region_update: Option<(Entity, Vec2, f32, f32, bool)> = None;

        {
            let mut writable_windows = windows.p1();
            let Ok((
                entity,
                mut window,
                mut window_transform,
                window_global,
                parent,
                gate,
                metrics,
                overflow_policy,
                tab_row_runtime,
            )) = writable_windows.get_mut(state.window_entity)
            else {
                commands
                    .entity(state.window_entity)
                    .remove::<WindowResizeInProgress>();
                active_interaction.state = None;
                return;
            };

            if !Self::window_interaction_allowed(entity, &window, gate, &interaction_state) {
                commands
                    .entity(state.window_entity)
                    .remove::<WindowResizeInProgress>();
                active_interaction.state = None;
                return;
            }

            let min_inner =
                Self::min_inner_constraints(&window, metrics, overflow_policy, tab_row_runtime);
            let max_inner = Self::max_inner_constraints(metrics, overflow_policy);

            let unclamped_width = match state.corner {
                ResizeCorner::BottomLeft => state.fixed_x_world - cursor_position.x,
                ResizeCorner::BottomRight => cursor_position.x - state.fixed_x_world,
            };
            let unclamped_height =
                state.fixed_top_y_world - window.header_height - cursor_position.y;
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
                let correction_world =
                    desired_center_world - window_global.translation().truncate();
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
                    window.has_close_button,
                ));
            }
        }

        if let Some((root_entity, correction_world)) = root_correction {
            if let Ok((mut root_transform, root_parent)) = windows.p2().get_mut(root_entity) {
                let correction_local = Self::world_delta_to_parent_local(
                    correction_world,
                    root_parent,
                    &parent_globals,
                );
                root_transform.translation.x += correction_local.x;
                root_transform.translation.y += correction_local.y;
            }
        }

        if let Some((root_entity, edge_center_local, new_width, header_height, has_close_button)) =
            drag_region_update
        {
            if let Ok(mut draggable) = draggables.get_mut(root_entity) {
                let (region, offset_x) =
                    Self::header_drag_region(new_width, header_height, has_close_button);
                draggable.region = Some(DraggableRegion {
                    region,
                    offset: edge_center_local + Vec2::new(offset_x, 0.0),
                });
            }
        }
    }

    fn route_explicit_window_content(
        mut commands: Commands,
        runtime_query: Query<&WindowScrollRuntime, With<Window>>,
        content_query: Query<(Entity, &WindowContent, Option<&ChildOf>)>,
    ) {
        for (entity, content, parent) in content_query.iter() {
            let Ok(runtime) = runtime_query.get(content.window_entity) else {
                continue;
            };
            if entity == content.window_entity
                || entity == runtime.vertical_root
                || entity == runtime.horizontal_root
                || entity == runtime.horizontal_proxy
                || entity == runtime.content_root
                || entity == runtime.vertical_bar
                || entity == runtime.horizontal_bar
            {
                continue;
            }
            if parent.is_some_and(|parent| parent.parent() == runtime.content_root) {
                continue;
            }
            // Explicit routing: only entities that opt in via `WindowContent`
            // are parented into the window's scroll content slot.
            commands.entity(runtime.content_root).add_child(entity);
        }
    }

    fn sync_scroll_runtime_geometry(
        windows: Query<(
            &Window,
            &WindowScrollRuntime,
            Option<&WindowContentRect>,
            Option<&WindowContentMetrics>,
        )>,
        mut root_query: Query<
            (&mut ScrollableViewport, &mut ScrollableContentExtent),
            With<ScrollableRoot>,
        >,
        mut bar_query: Query<&mut ScrollBar>,
        children_query: Query<&Children>,
        geometry_query: Query<(
            &GlobalTransform,
            Option<&Aabb>,
            Option<&InheritedVisibility>,
        )>,
    ) {
        fn accumulate_content_bounds(
            entity: Entity,
            root_inverse: Mat4,
            children_query: &Query<&Children>,
            geometry_query: &Query<(
                &GlobalTransform,
                Option<&Aabb>,
                Option<&InheritedVisibility>,
            )>,
            min_local: &mut Vec2,
            max_local: &mut Vec2,
            saw_any: &mut bool,
        ) {
            let Ok((global_transform, aabb, inherited_visibility)) = geometry_query.get(entity)
            else {
                if let Ok(children) = children_query.get(entity) {
                    for child in children.iter() {
                        accumulate_content_bounds(
                            child,
                            root_inverse,
                            children_query,
                            geometry_query,
                            min_local,
                            max_local,
                            saw_any,
                        );
                    }
                }
                return;
            };
            if inherited_visibility.is_some_and(|visibility| !visibility.get()) {
                return;
            }

            let matrix = global_transform.to_matrix();
            if let Some(aabb) = aabb {
                let center = Vec3::new(aabb.center.x, aabb.center.y, aabb.center.z);
                let half_x = aabb.half_extents.x.max(0.0);
                let half_y = aabb.half_extents.y.max(0.0);
                let corners = [
                    Vec3::new(-half_x, -half_y, 0.0),
                    Vec3::new(half_x, -half_y, 0.0),
                    Vec3::new(half_x, half_y, 0.0),
                    Vec3::new(-half_x, half_y, 0.0),
                ];
                for corner in corners {
                    let world = matrix.transform_point3(center + corner);
                    let local = root_inverse.transform_point3(world).truncate();
                    if !*saw_any {
                        *min_local = local;
                        *max_local = local;
                        *saw_any = true;
                    } else {
                        min_local.x = min_local.x.min(local.x);
                        min_local.y = min_local.y.min(local.y);
                        max_local.x = max_local.x.max(local.x);
                        max_local.y = max_local.y.max(local.y);
                    }
                }
            } else {
                let local = root_inverse
                    .transform_point3(global_transform.translation())
                    .truncate();
                if !*saw_any {
                    *min_local = local;
                    *max_local = local;
                    *saw_any = true;
                } else {
                    min_local.x = min_local.x.min(local.x);
                    min_local.y = min_local.y.min(local.y);
                    max_local.x = max_local.x.max(local.x);
                    max_local.y = max_local.y.max(local.y);
                }
            }

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {
                    accumulate_content_bounds(
                        child,
                        root_inverse,
                        children_query,
                        geometry_query,
                        min_local,
                        max_local,
                        saw_any,
                    );
                }
            }
        }

        fn measured_content_inner_size(
            content_root: Entity,
            children_query: &Query<&Children>,
            geometry_query: &Query<(
                &GlobalTransform,
                Option<&Aabb>,
                Option<&InheritedVisibility>,
            )>,
        ) -> Vec2 {
            let Ok((content_root_global, _, _)) = geometry_query.get(content_root) else {
                return Vec2::ZERO;
            };
            let Ok(children) = children_query.get(content_root) else {
                return Vec2::ZERO;
            };

            let root_inverse = content_root_global.to_matrix().inverse();
            let mut min_local = Vec2::ZERO;
            let mut max_local = Vec2::ZERO;
            let mut saw_any = false;
            for child in children.iter() {
                accumulate_content_bounds(
                    child,
                    root_inverse,
                    children_query,
                    geometry_query,
                    &mut min_local,
                    &mut max_local,
                    &mut saw_any,
                );
            }
            if !saw_any {
                return Vec2::ZERO;
            }
            (max_local - min_local).max(Vec2::ZERO)
        }

        for (window, runtime, content_rect, metrics) in windows.iter() {
            let inner_size = content_rect
                .map(|content_rect| content_rect.inner_size)
                .unwrap_or(window.boundary.dimensions)
                .max(Vec2::splat(1.0));
            // Runtime extents track actual composed content bounds first and
            // then respect optional explicit metrics as an override floor.
            let measured_inner =
                measured_content_inner_size(runtime.content_root, &children_query, &geometry_query);
            let preferred_inner = metrics
                .map(|metrics| metrics.preferred_inner.max(Vec2::ZERO))
                .unwrap_or(Vec2::ZERO)
                .max(measured_inner)
                .max(inner_size);

            if let Ok((mut viewport, mut extent)) = root_query.get_mut(runtime.vertical_root) {
                viewport.size = inner_size;
                extent.0 = preferred_inner.y;
            }
            if let Ok((mut viewport, mut extent)) = root_query.get_mut(runtime.horizontal_root) {
                viewport.size = inner_size;
                extent.0 = preferred_inner.x;
            }

            if let Ok(mut bar) = bar_query.get_mut(runtime.vertical_bar) {
                bar.track_color = window.boundary.color;
                bar.thumb_color = window.boundary.color;
            }
            if let Ok(mut bar) = bar_query.get_mut(runtime.horizontal_bar) {
                bar.track_color = window.boundary.color;
                bar.thumb_color = window.boundary.color;
            }
        }
    }

    fn sync_horizontal_proxy_offset(
        windows: Query<&WindowScrollRuntime>,
        horizontal_state_query: Query<&ScrollState, With<ScrollableRoot>>,
        mut proxy_query: Query<&mut Transform, With<WindowScrollHorizontalProxy>>,
    ) {
        for runtime in windows.iter() {
            let Ok(horizontal_state) = horizontal_state_query.get(runtime.horizontal_root) else {
                continue;
            };
            let Ok(mut proxy_transform) = proxy_query.get_mut(runtime.horizontal_proxy) else {
                continue;
            };
            proxy_transform.translation.x = -horizontal_state.offset_px;
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
                Option<&WindowTabRowRuntime>,
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
            for (entity, window, metrics, overflow_policy, tab_row_runtime, content_rect) in
                windows.p0().iter()
            {
                let min_inner =
                    Self::min_inner_constraints(window, metrics, overflow_policy, tab_row_runtime);
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

    fn sync_tab_row_layout(
        windows: Query<
            (&Window, &WindowTabRowRuntime),
            Or<(Changed<Window>, Changed<WindowTabRowRuntime>)>,
        >,
        target_query: Query<(Entity, &WindowTabRowTarget)>,
        mut transforms: Query<&mut Transform>,
    ) {
        for (window, runtime) in windows.iter() {
            let row_center_y = window.boundary.dimensions.y * 0.5 - runtime.row_height * 0.5;
            // `Table` rows are laid out from a top baseline (first row center at
            // `-row_height/2`), so the table transform must be set to the row's
            // top Y to visually align with center-positioned interaction targets.
            let row_top_y = row_center_y + runtime.row_height * 0.5;
            if let Ok(mut table_transform) = transforms.get_mut(runtime.table_entity) {
                table_transform.translation =
                    Vec3::new(-runtime.total_width * 0.5, row_top_y, runtime.z);
            }
            if let Ok(mut root_transform) = transforms.get_mut(runtime.tab_root) {
                root_transform.translation = Vec3::new(0.0, row_center_y, runtime.z + 0.02);
            }
            for (target_entity, target) in target_query.iter() {
                if target.tab_root != runtime.tab_root {
                    continue;
                }
                if let Ok(mut target_transform) = transforms.get_mut(target_entity) {
                    target_transform.translation = Vec3::new(
                        -runtime.total_width * 0.5
                            + runtime.tab_width * (target.index as f32 + 0.5),
                        row_center_y,
                        runtime.z + 0.02,
                    );
                }
            }
        }
    }

    fn sync_tab_row_visuals(
        tab_root_query: Query<
            (Entity, &TabBarState, &SelectableMenu),
            With<WindowTabRowInteractionRoot>,
        >,
        runtime_query: Query<&WindowTabRowRuntime, With<WindowTabRow>>,
        target_query: Query<(&WindowTabRowTarget, &Hoverable)>,
        mut table_query: Query<(Entity, &WindowTabRowTable, &mut Table)>,
        table_children_query: Query<&Children>,
        column_children_query: Query<&Children, With<Column>>,
        cell_children_query: Query<&Children>,
        mut cell_query: Query<&mut Cell>,
        mut text_query: Query<(&mut TextColor, &mut TextFont, &mut Transform)>,
    ) {
        let mut state_by_root: HashMap<Entity, (usize, usize)> = HashMap::new();
        for (root_entity, tab_state, selectable_menu) in tab_root_query.iter() {
            state_by_root.insert(
                root_entity,
                (tab_state.active_index, selectable_menu.selected_index),
            );
        }

        let mut style_by_root: HashMap<Entity, (Color, f32, f32, f32)> = HashMap::new();
        for runtime in runtime_query.iter() {
            style_by_root.insert(
                runtime.tab_root,
                (
                    runtime.color,
                    runtime.text_size,
                    runtime.selected_text_size,
                    runtime.z + 0.1,
                ),
            );
        }

        let mut hovered_by_root_index: HashMap<(Entity, usize), bool> = HashMap::new();
        for (target, hoverable) in target_query.iter() {
            if hoverable.hovered {
                hovered_by_root_index.insert((target.tab_root, target.index), true);
            }
        }

        for (table_entity, table_binding, mut table) in table_query.iter_mut() {
            let Some((active_index, selected_index)) =
                state_by_root.get(&table_binding.tab_root).copied()
            else {
                continue;
            };
            let Some((tab_color, text_size, selected_text_size, text_z)) =
                style_by_root.get(&table_binding.tab_root).copied()
            else {
                continue;
            };
            if table.columns.is_empty() {
                continue;
            }

            let active_index = active_index.min(table.columns.len() - 1);
            let selected_index = selected_index.min(table.columns.len() - 1);
            for (column_index, column) in table.columns.iter_mut().enumerate() {
                column.cell_boundary_sides = Some(RectangleSides {
                    top: true,
                    bottom: column_index != active_index,
                    left: true,
                    right: true,
                });
                column.cell_boundary_color = Some(tab_color);
            }

            let Ok(column_entities) = table_children_query.get(table_entity) else {
                continue;
            };
            for (column_index, column_entity) in column_entities.iter().enumerate() {
                let Ok(cells) = column_children_query.get(column_entity) else {
                    continue;
                };
                let Some(cell_entity) = cells.first() else {
                    continue;
                };

                let highlighted = hovered_by_root_index
                    .get(&(table_binding.tab_root, column_index))
                    .copied()
                    .unwrap_or(column_index == selected_index);
                let open = column_index == active_index;
                if let Ok(mut cell) = cell_query.get_mut(*cell_entity) {
                    cell.set_fill_color(if highlighted { tab_color } else { Color::BLACK });
                }

                let Ok(cell_children) = cell_children_query.get(*cell_entity) else {
                    continue;
                };
                for child in cell_children.iter() {
                    let Ok((mut color, mut font, mut transform)) = text_query.get_mut(child) else {
                        continue;
                    };
                    if highlighted {
                        color.0 = Color::BLACK;
                        font.font_size = selected_text_size;
                        font.weight = FontWeight::BOLD;
                    } else if open {
                        color.0 = tab_color;
                        font.font_size = selected_text_size;
                        font.weight = FontWeight::BOLD;
                    } else {
                        color.0 = tab_color;
                        font.font_size = text_size;
                        font.weight = FontWeight::NORMAL;
                    }
                    transform.translation.z = text_z;
                    break;
                }
            }
        }
    }

    fn sync_root_drag_bounds(
        mut commands: Commands,
        window: Single<&bevy::window::Window, With<PrimaryWindow>>,
        offscreen_camera_query: Query<(&Camera, &GlobalTransform), With<OffscreenCamera>>,
        windows: Query<(Entity, &Window, &GlobalTransform)>,
        root_globals: Query<&GlobalTransform>,
        mut root_bounds_query: Query<&mut DraggableViewportBounds>,
        existing_bounds: Query<Entity, With<DraggableViewportBounds>>,
    ) {
        let Ok((camera, camera_transform)) = offscreen_camera_query.single() else {
            return;
        };
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

            let min_window_center = Vec2::new(viewport_min.x + half_w, viewport_min.y + half_h);
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
            if let Ok(mut existing_bounds) = root_bounds_query.get_mut(*root_entity) {
                *existing_bounds = *bounds;
            } else {
                commands.entity(*root_entity).insert(*bounds);
            }
        }

        for entity in existing_bounds.iter() {
            if !bounds_by_root.contains_key(&entity) {
                commands.entity(entity).remove::<DraggableViewportBounds>();
            }
        }
    }

    fn clamp_to_viewport(
        window: Single<&bevy::window::Window, With<PrimaryWindow>>,
        offscreen_camera_query: Query<(&Camera, &GlobalTransform), With<OffscreenCamera>>,
        windows: Query<(Entity, &Window, &GlobalTransform)>,
        mut root_transforms: Query<(&mut Transform, Option<&ChildOf>)>,
        parent_globals: Query<&GlobalTransform>,
    ) {
        let Ok((camera, camera_transform)) = offscreen_camera_query.single() else {
            return;
        };
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
            let world = camera.viewport_to_world_2d(camera_transform, corner).ok()?;
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
        tab_row_runtime: Option<&WindowTabRowRuntime>,
    ) -> Vec2 {
        let mut min_inner = Vec2::new(
            WINDOW_MIN_WIDTH.max(window.header_height + 10.0),
            WINDOW_MIN_HEIGHT,
        );
        if let Some(tab_row_runtime) = tab_row_runtime {
            min_inner.x = min_inner.x.max(tab_row_runtime.total_width);
        }
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

    fn interaction_owner(window_entity: Entity, window: &Window) -> Entity {
        window.root_entity.unwrap_or(window_entity)
    }

    fn owner_base_layer_active(interaction_state: &UiInteractionState, owner: Entity) -> bool {
        interaction_state
            .active_layers_by_owner
            .get(&owner)
            .is_none_or(|active| active.kind == UiLayerKind::Base)
    }

    fn window_interaction_allowed(
        window_entity: Entity,
        window: &Window,
        policy: Option<&UiInputPolicy>,
        interaction_state: &UiInteractionState,
    ) -> bool {
        let owner = Self::interaction_owner(window_entity, window);
        ui_input_policy_allows_mode(policy, interaction_state.input_mode_for_owner(owner))
            && scoped_owner_has_focus(Some(owner), interaction_state.focused_owner)
            && Self::owner_base_layer_active(interaction_state, owner)
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
                            font_size: scaled_font_size(12.0),
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

    fn spawn_scroll_runtime(
        commands: &mut Commands,
        window_entity: Entity,
        owner_entity: Entity,
        inner_size: Vec2,
        color: Color,
        gate: Option<UiInputPolicy>,
    ) -> WindowScrollRuntime {
        let mut vertical_root = None;
        let mut vertical_content = None;
        let mut horizontal_root = None;
        let mut horizontal_proxy = None;
        let mut content_root = None;
        let mut vertical_bar = None;
        let mut horizontal_bar = None;

        commands.entity(window_entity).with_children(|parent| {
            let inner_size = inner_size.max(Vec2::splat(1.0));
            let v_root = parent
                .spawn((
                    Name::new("window_scroll_vertical_root"),
                    WindowScrollManaged,
                    ScrollableRoot::new(owner_entity, ScrollAxis::Vertical),
                    ScrollZoomConfig {
                        enabled: true,
                        ..default()
                    },
                    ScrollableViewport::new(inner_size),
                    ScrollableContentExtent(inner_size.y),
                    Transform::from_xyz(0.0, 0.0, 0.05),
                ))
                .id();
            vertical_root = Some(v_root);
            if let Some(gate) = gate {
                parent.commands().entity(v_root).insert(gate);
            }

            let v_content = parent
                .spawn((
                    Name::new("window_scroll_vertical_content"),
                    WindowScrollManaged,
                    ScrollableContent,
                    Transform::default(),
                ))
                .id();
            vertical_content = Some(v_content);

            let h_root = parent
                .spawn((
                    Name::new("window_scroll_horizontal_root"),
                    WindowScrollManaged,
                    ScrollableRoot::new(owner_entity, ScrollAxis::Horizontal)
                        .with_backend(ScrollBackend::StateOnly),
                    ScrollZoomConfig {
                        enabled: true,
                        ..default()
                    },
                    ScrollableViewport::new(inner_size),
                    ScrollableContentExtent(inner_size.x),
                    Transform::default(),
                ))
                .id();
            horizontal_root = Some(h_root);
            if let Some(gate) = gate {
                parent.commands().entity(h_root).insert(gate);
            }

            let h_proxy = parent
                .spawn((
                    Name::new("window_scroll_horizontal_proxy"),
                    WindowScrollManaged,
                    WindowScrollHorizontalProxy,
                    Transform::default(),
                ))
                .id();
            horizontal_proxy = Some(h_proxy);

            let content = parent
                .spawn((
                    Name::new("window_scroll_content_root"),
                    WindowScrollManaged,
                    Transform::default(),
                ))
                .id();
            content_root = Some(content);

            let mut v_bar = ScrollBar::new(v_root);
            v_bar.width = 10.0;
            v_bar.margin = 4.0;
            v_bar.track_color = color;
            v_bar.thumb_color = color;
            let v_bar_entity = parent
                .spawn((
                    Name::new("window_scroll_vertical_bar"),
                    WindowScrollManaged,
                    v_bar,
                    Transform::from_xyz(0.0, 0.0, 0.2),
                ))
                .id();
            vertical_bar = Some(v_bar_entity);

            let mut h_bar = ScrollBar::new(h_root);
            h_bar.parent_override = Some(v_root);
            h_bar.width = 10.0;
            h_bar.margin = 4.0;
            h_bar.track_color = color;
            h_bar.thumb_color = color;
            let h_bar_entity = parent
                .spawn((
                    Name::new("window_scroll_horizontal_bar"),
                    WindowScrollManaged,
                    h_bar,
                    Transform::from_xyz(0.0, 0.0, 0.2),
                ))
                .id();
            horizontal_bar = Some(h_bar_entity);
        });

        let vertical_root = vertical_root.expect("window vertical scroll root");
        let vertical_content = vertical_content.expect("window vertical scroll content");
        let horizontal_root = horizontal_root.expect("window horizontal scroll root");
        let horizontal_proxy = horizontal_proxy.expect("window horizontal scroll proxy");
        let content_root = content_root.expect("window scroll content root");
        let vertical_bar = vertical_bar.expect("window vertical scroll bar");
        let horizontal_bar = horizontal_bar.expect("window horizontal scroll bar");

        commands.entity(vertical_root).add_child(vertical_content);
        commands
            .entity(vertical_content)
            .add_child(horizontal_proxy);
        commands.entity(horizontal_proxy).add_child(content_root);

        WindowScrollRuntime {
            vertical_root,
            horizontal_root,
            horizontal_proxy,
            content_root,
            vertical_bar,
            horizontal_bar,
        }
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
                        let region = Some(close_button_click_region(close_boundary.dimensions.x));
                        click.region = region;
                        color_anchor.0 = close_boundary.color;
                        palette.idle_color = close_boundary.color;
                    }
                }
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (boundary, header_h, close_btn, root_entity, gate) = {
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
                entity_mut.get::<UiInputPolicy>().copied(),
            )
        };
        let resolved_gate = gate.or_else(|| {
            root_entity
                .filter(|root| *root != entity)
                .and_then(|root| world.entity(root).get::<UiInputPolicy>().copied())
        });

        if !world.entity(entity).contains::<WindowOverflowPolicy>() {
            world
                .commands()
                .entity(entity)
                .insert(WindowOverflowPolicy::AllowOverflow);
        }
        if !world.entity(entity).contains::<WindowContentMetrics>() {
            world
                .commands()
                .entity(entity)
                .insert(WindowContentMetrics::from_min_inner(boundary.dimensions));
        }

        if let Some(root_entity) = root_entity {
            world
                .commands()
                .entity(root_entity)
                .insert(SelectableScopeOwner::new(root_entity));
        }

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
                let mut close_button = parent.spawn((
                    WindowCloseButton { root_entity },
                    Transform::from_xyz(
                        (boundary.dimensions.x - header_h) / 2.0,
                        (boundary.dimensions.y + header_h) / 2.0,
                        0.0,
                    ),
                ));
                if let Some(root_entity) = root_entity {
                    close_button.insert(SelectableScopeOwner::new(root_entity));
                }
                if let Some(gate) = resolved_gate {
                    close_button.insert(gate);
                }
                close_button_entity = Some(close_button.id());
            }
        });

        world.commands().entity(entity).insert(WindowParts {
            body: body_entity,
            header: header_entity,
            close_button: close_button_entity,
            ..default()
        });

        if !world.entity(entity).contains::<WindowScrollRuntime>() {
            let owner_entity = root_entity.unwrap_or(entity);
            let runtime = Self::spawn_scroll_runtime(
                &mut world.commands(),
                entity,
                owner_entity,
                boundary.dimensions,
                boundary.color,
                resolved_gate,
            );
            world.commands().entity(entity).insert(runtime);
        }

        if let Some(root_entity) = root_entity {
            if let Some(mut draggable) = world.entity_mut(root_entity).get_mut::<Draggable>() {
                let (region, offset_x) =
                    Self::header_drag_region(boundary.dimensions.x, header_h, close_btn);
                draggable.region = Some(DraggableRegion {
                    region,
                    offset: Vec2::new(offset_x, (boundary.dimensions.y + header_h) * 0.5),
                });
            }
        }
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
        let Some(button) = world.entity(entity).get::<WindowCloseButton>() else {
            return;
        };
        let root = button.root_entity;
        let gate = world
            .entity(entity)
            .get::<UiInputPolicy>()
            .copied()
            .or_else(|| {
                world.entity(entity).get::<ChildOf>().and_then(|parent| {
                    world
                        .entity(parent.parent())
                        .get::<UiInputPolicy>()
                        .copied()
                })
            })
            .or_else(|| {
                root.and_then(|root| world.entity(root).get::<UiInputPolicy>().copied())
            });
        let click_sound = world
            .get_resource::<AssetServer>()
            .map(|assets| assets.load("./audio/effects/mouse_click.ogg"));

        world.commands().entity(entity).with_children(|parent| {
            let default_region = close_button_click_region(20.0);
            let default_icon_dim = Vec2::splat(10.0);
            parent.spawn((WindowCloseButtonBorder, BorderedRectangle::default()));

            let close_actions = if click_sound.is_some() {
                vec![
                    InputAction::PlaySound(WindowSounds::Close),
                    InputAction::Despawn(root),
                ]
            } else {
                vec![InputAction::Despawn(root)]
            };

            let mut icon_commands = parent.spawn((
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
                    WindowActions::CloseWindow => close_actions.clone()
                )),
            ));
            if let Some(gate) = gate {
                icon_commands.insert(gate);
            }
            if let Some(root) = root {
                icon_commands.insert(SelectableScopeOwner::new(root));
            }
            let icon_entity = icon_commands.id();

            if let Some(handle) = click_sound.clone() {
                parent
                    .commands()
                    .entity(icon_entity)
                    .insert(TransientAudioPallet::new(vec![(
                        WindowSounds::Close,
                        vec![TransientAudio::new(handle, 0.1, true, 1.0, true)],
                    )]));
            }
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

// New UI-primitive naming, kept as aliases during migration.
pub use self::Window as UiWindow;
pub use self::WindowActions as UiWindowActions;
pub use self::WindowContent as UiWindowContent;
pub use self::WindowContentHost as UiWindowContentHost;
pub use self::WindowContentMetrics as UiWindowContentMetrics;
pub use self::WindowOverflowPolicy as UiWindowOverflowPolicy;
pub use self::WindowPlugin as UiWindowPlugin;
pub use self::WindowResizeInProgress as UiWindowResizeInProgress;
pub use self::WindowSounds as UiWindowSounds;
pub use self::WindowSystem as UiWindowSystem;
pub use self::WindowTabRow as UiWindowTabRow;
pub use self::WindowTitle as UiWindowTitle;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_window_on_insert_sets_drag_region_for_draggable_root() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();

        let window_entity = app
            .world_mut()
            .spawn((
                Draggable::default(),
                UiWindow::new(
                    None,
                    HollowRectangle {
                        dimensions: Vec2::new(160.0, 90.0),
                        ..default()
                    },
                    24.0,
                    false,
                    None,
                ),
            ))
            .id();

        app.update();

        let draggable = app
            .world()
            .get::<Draggable>(window_entity)
            .expect("draggable");
        let region = draggable.region.as_ref().expect("drag region");
        assert!(region.region.x > 160.0);
        assert!(region.region.y > 24.0);
        assert!(region.offset.y > 0.0);
    }

    #[test]
    fn ui_window_with_close_button_spawns_clickable_close_action() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<bevy::audio::AudioSource>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();

        app.world_mut().spawn(UiWindow::new(
            None,
            HollowRectangle {
                dimensions: Vec2::new(120.0, 80.0),
                ..default()
            },
            20.0,
            true,
            None,
        ));
        app.update();

        let has_close_clickable = {
            let world = app.world_mut();
            let mut query = world.query::<&Clickable<UiWindowActions>>();
            query
                .iter(world)
                .any(|clickable| clickable.actions.contains(&UiWindowActions::CloseWindow))
        };
        assert!(has_close_clickable);
    }

    #[test]
    fn ui_window_close_button_clickable_inherits_window_interaction_gate() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<bevy::audio::AudioSource>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();

        app.world_mut().spawn((
            UiInputPolicy::CapturedOnly,
            UiWindow::new(
                None,
                HollowRectangle {
                    dimensions: Vec2::new(120.0, 80.0),
                    ..default()
                },
                20.0,
                true,
                None,
            ),
        ));
        app.update();

        let has_gated_close_clickable = {
            let world = app.world_mut();
            let mut query = world.query::<(&Clickable<UiWindowActions>, &UiInputPolicy)>();
            query.iter(world).any(|(clickable, gate)| {
                clickable.actions.contains(&UiWindowActions::CloseWindow)
                    && *gate == UiInputPolicy::CapturedOnly
            })
        };
        assert!(has_gated_close_clickable);
    }

    #[test]
    fn ui_window_on_insert_seeds_default_scroll_runtime() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<bevy::audio::AudioSource>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();

        let window_entity = app
            .world_mut()
            .spawn(UiWindow::new(
                None,
                HollowRectangle {
                    dimensions: Vec2::new(180.0, 120.0),
                    ..default()
                },
                20.0,
                true,
                None,
            ))
            .id();

        app.update();

        let overflow_policy = app
            .world()
            .get::<UiWindowOverflowPolicy>(window_entity)
            .copied()
            .expect("window overflow policy");
        assert_eq!(overflow_policy, UiWindowOverflowPolicy::AllowOverflow);
        assert!(app
            .world()
            .get::<UiWindowContentMetrics>(window_entity)
            .is_some());

        let runtime = app
            .world()
            .get::<WindowScrollRuntime>(window_entity)
            .copied()
            .expect("window scroll runtime");
        assert!(app
            .world()
            .entity(runtime.vertical_root)
            .contains::<ScrollableRoot>());
        assert!(app
            .world()
            .entity(runtime.horizontal_root)
            .contains::<ScrollableRoot>());
        assert!(app
            .world()
            .entity(runtime.horizontal_proxy)
            .contains::<WindowScrollHorizontalProxy>());
        assert!(app
            .world()
            .entity(runtime.content_root)
            .contains::<WindowScrollManaged>());
        assert!(app
            .world()
            .entity(runtime.vertical_bar)
            .contains::<ScrollBar>());
        assert!(app
            .world()
            .entity(runtime.horizontal_bar)
            .contains::<ScrollBar>());
    }
}
