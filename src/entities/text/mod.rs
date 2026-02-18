use bevy::{
    camera::primitives::Aabb,
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
    sprite::Anchor,
    text::{TextBounds, TextLayoutInfo},
};
use serde::Deserialize;

use crate::{
    entities::sprites::{
        compound::{BorderedRectangle, HollowRectangle, RectangleSides},
        window::{
            Window, WindowContentHost, WindowContentMetrics, WindowOverflowPolicy,
            WindowResizeInProgress, WindowSystem, WindowTitle,
        },
        SpritePlugin,
    },
    systems::{
        colors::{ColorAnchor, CLICKED_BUTTON, HOVERED_BUTTON, PRIMARY_COLOR},
        interaction::{
            Clickable, Draggable, DraggableRegion, InteractionVisualPalette,
            InteractionVisualState, KeyMapping, Pressable,
        },
        time::Dilation,
    },
};

pub struct TextPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum TextSystem {
    Contracts,
    ContentLayout,
    TableLayout,
}

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                TextSystem::Contracts,
                TextSystem::ContentLayout.after(TextSystem::Contracts),
                TextSystem::TableLayout.after(TextSystem::ContentLayout),
            ),
        )
        .add_systems(
            Update,
            (
                TextWindow::refresh_layout_state,
                TextWindow::sync_window_contract,
            )
                .chain()
                .in_set(TextSystem::Contracts)
                .after(WindowSystem::Layout),
        )
        .add_systems(
            Update,
            (
                TextWindow::apply_window_to_text,
                WindowedTable::apply_window_to_table,
            )
                .in_set(TextSystem::ContentLayout)
                .after(WindowSystem::Layout),
        )
        .add_systems(
            Update,
            (
                Table::propagate_changes,
                Column::propagate_changes,
                Cell::propagate_changes,
                Cell::recenter_text_vertically,
            )
                .chain()
                .in_set(TextSystem::TableLayout),
        );

        if !app.is_plugin_added::<SpritePlugin>() {
            app.add_plugins(SpritePlugin);
        }
    }
}

fn default_font() -> TextFont {
    TextFont {
        font_size: scaled_font_size(12.0),
        ..default()
    }
}

fn default_font_color() -> TextColor {
    TextColor(PRIMARY_COLOR)
}

fn default_text_layout() -> TextLayout {
    TextLayout {
        justify: Justify::Center,
        ..default()
    }
}

fn default_nowrap_layout() -> TextLayout {
    TextLayout {
        justify: Justify::Center,
        linebreak: LineBreak::NoWrap,
        ..default()
    }
}

// Existing components
#[derive(Component)]
#[require(TextFont = default_font(), TextColor = default_font_color(), TextLayout = default_text_layout())]
pub struct TextRaw;

impl Default for TextRaw {
    fn default() -> Self {
        Self
    }
}

#[derive(Component)]
#[require(TextFont = default_font(), TextColor = default_font_color(), TextLayout = default_nowrap_layout())]
pub struct TextSprite;

impl Default for TextSprite {
    fn default() -> Self {
        Self
    }
}

#[derive(Component)]
#[require(TextFont = default_font(), TextColor = default_font_color(), TextLayout = default_nowrap_layout())]
pub struct TextTitle;

impl Default for TextTitle {
    fn default() -> Self {
        Self
    }
}

#[derive(Component, Deserialize, Clone)]
pub struct TextFrames {
    pub frames: Vec<String>,
}

impl Default for TextFrames {
    fn default() -> Self {
        Self { frames: vec![] }
    }
}

impl TextFrames {
    pub fn new(frames: Vec<String>) -> Self {
        Self { frames }
    }

    pub fn load(input_string: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match serde_json::from_str(input_string) {
            Ok(frames) => Ok(Self::new(frames)),
            Err(e) => {
                eprintln!("Failed to deserialize JSON from file: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
}

const BASE_CHAR_WIDTH: f32 = 7.0;
const BASE_LINE_HEIGHT: f32 = 16.0;
const BASE_TEXT_WIDTH_PER_CHAR: f32 = 7.92;
const BASE_TEXT_HEIGHT_PER_LINE: f32 = 12.0;
pub const GLOBAL_TEXT_SCALE: f32 = 1.5;

#[inline]
pub const fn scaled_font_size(size: f32) -> f32 {
    size * GLOBAL_TEXT_SCALE
}

#[inline]
pub const fn scaled_text_units(units: f32) -> f32 {
    units * GLOBAL_TEXT_SCALE
}

const CHAR_WIDTH: f32 = scaled_text_units(BASE_CHAR_WIDTH);
const LINE_HEIGHT: f32 = scaled_text_units(BASE_LINE_HEIGHT);

// ────────────────────────────────────────────────────────────────
//  CharacterSprite now carries figure dimensions so `offset()` works
// ────────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct CharacterSprite {
    pub row: usize,
    pub col: usize,
    pub cols: usize,
    pub lines: usize,
}

impl CharacterSprite {
    fn offset(&self) -> Vec3 {
        let x = (self.col as f32 - (self.cols as f32 - 1.) * 0.5) * CHAR_WIDTH;
        let y = ((self.lines - 1 - self.row) as f32) * LINE_HEIGHT;
        Vec3::new(x, y, 0.0)
    }
}

// ────────────────────────────────────────────────────────────────
//  NEW  GlyphString component – spawns glyphs + AABB on insert
// ────────────────────────────────────────────────────────────────
#[derive(Component)]
#[component(on_insert = GlyphString::on_insert)]
#[require(Transform, Visibility)]
pub struct GlyphString {
    pub text: String,
    pub depth: f32,
}

impl GlyphString {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let glyph_string = world.entity(entity).get::<GlyphString>().unwrap();

        let text = glyph_string.text.clone();
        let depth = glyph_string.depth.clone();

        let lines: Vec<&str> = text.lines().collect();
        let lines_count = lines.len();
        let max_cols = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

        // 1. spawn each glyph as a child
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                world.commands().entity(entity).with_children(|p| {
                    p.spawn((
                        CharacterSprite {
                            row,
                            col,
                            cols: max_cols,
                            lines: lines_count,
                        },
                        TextSprite,
                        Text2d::new(ch.to_string()),
                        Anchor::BOTTOM_CENTER,
                        Transform::from_translation(
                            CharacterSprite {
                                row,
                                col,
                                cols: max_cols,
                                lines: lines_count,
                            }
                            .offset(),
                        ),
                    ));
                });
            }
        }

        // 2. add one parent-level AABB
        let half_x = max_cols as f32 * CHAR_WIDTH * 0.5;
        let half_y = lines_count as f32 * LINE_HEIGHT * 0.5;
        let half_z = depth * 0.5;
        let centre = Vec3::new(0.0, half_y, 0.0);

        world.commands().entity(entity).insert(Aabb {
            center: centre.into(),
            half_extents: Vec3::new(half_x, half_y, half_z).into(),
        });
    }
}

#[derive(Component)]
#[require(TextSprite, Text2d, TextFrames)]
pub struct Animated {
    pub current_frame: usize,
    pub timer: Timer,
}

impl Animated {
    pub fn new(current_frame: usize, time_seconds: f32) -> Self {
        Self {
            current_frame,
            timer: Timer::from_seconds(time_seconds, TimerMode::Repeating),
        }
    }
}

impl Default for Animated {
    fn default() -> Self {
        Self {
            current_frame: 0,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

impl Animated {
    pub fn animate_text(
        time: Res<Time>,
        dilation: Res<Dilation>,
        mut query: Query<(&mut Animated, &TextFrames, &mut Text2d)>,
    ) {
        for (mut animation, frames, mut text) in query.iter_mut() {
            animation.timer.tick(time.delta().mul_f32(dilation.0));
            if animation.timer.just_finished() {
                animation.current_frame = (animation.current_frame + 1) % frames.frames.len();
                text.0 = frames.frames[animation.current_frame].clone();
            }
        }
    }
}

pub fn get_text_width(text: &String) -> f32 {
    let text_length = text
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_else(|| text.chars().count());
    text_length as f32 * scaled_text_units(BASE_TEXT_WIDTH_PER_CHAR)
}

pub fn get_text_height(text: &String) -> f32 {
    text.lines().count() as f32 * scaled_text_units(BASE_TEXT_HEIGHT_PER_LINE)
}

#[derive(Component)]
#[require(
    Text2d = default_button_text(), 
    TextFont = default_button_font(), 
    TextColor = default_font_color(),
    TextBounds = default_button_bounds(),
    ColorAnchor,
    InteractionVisualState,
    InteractionVisualPalette = default_button_visual_palette()
)]

pub struct TextButton;

fn default_button_text() -> Text2d {
    Text2d::new("Default Button Text")
}

fn default_button_font() -> TextFont {
    TextFont {
        font_size: scaled_font_size(16.0),
        ..default()
    }
}

pub fn default_button_bounds() -> TextBounds {
    TextBounds {
        width: Some(1000.0),
        ..default()
    }
}

fn default_button_visual_palette() -> InteractionVisualPalette {
    InteractionVisualPalette::new(
        default_font_color().0,
        HOVERED_BUTTON,
        CLICKED_BUTTON,
        HOVERED_BUTTON,
    )
}

impl TextButton {
    pub fn new<T>(actions: Vec<T>, keys: Vec<KeyCode>, text: impl Into<String>) -> impl Bundle
    where
        T: Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + std::cmp::Eq
            + Send
            + Sync
            + 'static,
    {
        (
            TextButton,
            TextLayout {
                justify: Justify::Center,
                ..default()
            },
            Clickable::new(actions.clone()),
            Pressable::new(vec![KeyMapping {
                keys,
                actions,
                allow_repeated_activation: false,
            }]),
            Text2d::new(text.into()),
        )
    }
}

fn get_anchor_offset(anchor: &Anchor, dimensions: Vec2) -> Vec2 {
    let width = dimensions.x;
    let height = dimensions.y;

    match anchor {
        &Anchor::TOP_LEFT => Vec2::new(-width / 2.0, height / 2.0),
        &Anchor::TOP_CENTER => Vec2::new(0.0, height / 2.0),
        &Anchor::TOP_RIGHT => Vec2::new(width / 2.0, height / 2.0),
        &Anchor::CENTER_LEFT => Vec2::new(-width / 2.0, 0.0),
        &Anchor::CENTER => Vec2::new(0.0, 0.0),
        &Anchor::CENTER_RIGHT => Vec2::new(width / 2.0, 0.0),
        &Anchor::BOTTOM_LEFT => Vec2::new(-width / 2.0, -height / 2.0),
        &Anchor::BOTTOM_CENTER => Vec2::new(0.0, -height / 2.0),
        &Anchor::BOTTOM_RIGHT => Vec2::new(width / 2.0, -height / 2.0),
        &Anchor(offset) => offset,
    }
}

fn justify_for_anchor(anchor: &Anchor) -> Justify {
    match anchor {
        &Anchor::TOP_LEFT | &Anchor::CENTER_LEFT | &Anchor::BOTTOM_LEFT => Justify::Left,
        &Anchor::TOP_CENTER | &Anchor::CENTER | &Anchor::BOTTOM_CENTER => Justify::Center,
        &Anchor::TOP_RIGHT | &Anchor::CENTER_RIGHT | &Anchor::BOTTOM_RIGHT => Justify::Right,
        _ => Justify::Center,
    }
}

#[derive(Clone, Component)]
#[require(Transform, Visibility)]
#[component(on_insert = TextWindow::on_insert)]
pub struct TextWindow {
    pub title: Option<WindowTitle>,
    pub border_color: Color,
    pub header_height: f32,
    pub padding: Vec2,
    pub has_close_button: bool,
}

impl Default for TextWindow {
    fn default() -> Self {
        Self {
            title: None,
            border_color: PRIMARY_COLOR,
            header_height: 20.0,
            padding: Vec2::new(20.0, 10.0),
            has_close_button: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
struct TextWindowLayoutState {
    base_content_size: Vec2,
    min_content_size: Vec2,
}

#[derive(Component, Clone, Copy)]
struct TextWindowBoundsPolicy {
    constrain_width: bool,
    constrain_height: bool,
}

impl TextWindow {
    fn measured_text_size(text_layout: &TextLayoutInfo, text_bounds: &TextBounds) -> Vec2 {
        let width = text_bounds.width.unwrap_or(text_layout.size.x);
        let height = text_bounds.height.unwrap_or(text_layout.size.y);
        Vec2::new(width, height)
    }

    fn refresh_layout_state(
        mut text_windows: Query<
            (
                &TextLayoutInfo,
                &TextBounds,
                &TextWindowBoundsPolicy,
                &mut TextWindowLayoutState,
            ),
            (
                With<TextWindow>,
                Or<(Changed<TextLayoutInfo>, Changed<TextBounds>)>,
            ),
        >,
    ) {
        for (text_layout, text_bounds, bounds_policy, mut layout_state) in text_windows.iter_mut() {
            let measured_size =
                Self::measured_text_size(text_layout, text_bounds).max(Vec2::splat(1.0));
            let target_min_size = Vec2::new(
                if bounds_policy.constrain_width {
                    layout_state.base_content_size.x
                } else {
                    measured_size.x.max(layout_state.base_content_size.x)
                },
                if bounds_policy.constrain_height {
                    layout_state.base_content_size.y
                } else {
                    measured_size.y.max(layout_state.base_content_size.y)
                },
            );

            if (layout_state.min_content_size - target_min_size).length_squared() > 0.0001 {
                layout_state.min_content_size = target_min_size;
            }
        }
    }

    fn sync_window_contract(
        text_windows: Query<
            (&TextWindow, &TextWindowLayoutState, &WindowContentHost),
            Or<(
                Added<TextWindowLayoutState>,
                Changed<TextWindowLayoutState>,
                Changed<TextWindow>,
            )>,
        >,
        mut window_contracts: Query<&mut WindowContentMetrics>,
    ) {
        for (text_window, layout_state, window_host) in text_windows.iter() {
            let min_inner =
                (layout_state.min_content_size + text_window.padding).max(Vec2::splat(1.0));
            if let Ok(mut metrics) = window_contracts.get_mut(window_host.window_entity) {
                if (metrics.min_inner - min_inner).length_squared() > 0.0001 {
                    metrics.min_inner = min_inner;
                }
                if (metrics.preferred_inner - min_inner).length_squared() > 0.0001 {
                    metrics.preferred_inner = min_inner;
                }
            }
        }
    }

    fn apply_window_to_text(
        mut text_windows: Query<(
            &TextWindow,
            &TextWindowBoundsPolicy,
            Option<&Anchor>,
            Option<&mut Draggable>,
            &WindowContentHost,
            &mut TextBounds,
        )>,
        windows: Query<(&Window, Option<&WindowResizeInProgress>)>,
        changed_windows: Query<(), Changed<Window>>,
        mut window_transforms: Query<&mut Transform, With<Window>>,
    ) {
        for (text_window, bounds_policy, anchor, draggable, window_host, mut text_bounds) in
            text_windows.iter_mut()
        {
            if changed_windows.get(window_host.window_entity).is_err() {
                continue;
            }
            let Ok((window, resize_in_progress)) = windows.get(window_host.window_entity) else {
                continue;
            };

            let inner_size = window.boundary.dimensions.max(Vec2::splat(1.0));
            let content_size = (inner_size - text_window.padding).max(Vec2::splat(1.0));

            if bounds_policy.constrain_width {
                let width_changed = text_bounds
                    .width
                    .is_none_or(|width| (width - content_size.x).abs() > 0.01);
                if width_changed {
                    text_bounds.width = Some(content_size.x);
                }
            }
            if bounds_policy.constrain_height {
                let height_changed = text_bounds
                    .height
                    .is_none_or(|height| (height - content_size.y).abs() > 0.01);
                if height_changed {
                    text_bounds.height = Some(content_size.y);
                }
            }

            let anchor = anchor.copied().unwrap_or(Anchor::CENTER);
            let anchor_offset = get_anchor_offset(&anchor, content_size).extend(0.1);

            if let Ok(mut window_transform) = window_transforms.get_mut(window_host.window_entity) {
                if resize_in_progress.is_none() {
                    window_transform.translation.x = -anchor_offset.x;
                    window_transform.translation.y = -anchor_offset.y;
                } else {
                    // Keep horizontal anchoring stable for text while preserving
                    // manual vertical placement from active corner-resize.
                    window_transform.translation.x = -anchor_offset.x;
                }
            }

            if let Some(mut draggable) = draggable {
                if resize_in_progress.is_none() {
                    let edge_padding = 10.0;
                    draggable.region = Some(DraggableRegion {
                        region: Vec2::new(
                            inner_size.x + edge_padding,
                            window.header_height + edge_padding,
                        ),
                        offset: Vec2::new(
                            -anchor_offset.x,
                            (inner_size.y + window.header_height) * 0.5 - anchor_offset.y,
                        ),
                    });
                }
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (title, header_height, color, has_close_button, padding) = {
            if let Some(window) = world.entity(entity).get::<TextWindow>() {
                (
                    window.title.clone(),
                    window.header_height,
                    window.border_color,
                    window.has_close_button,
                    window.padding,
                )
            } else {
                return;
            }
        };

        let anchor = world
            .entity(entity)
            .get::<Anchor>()
            .copied()
            .unwrap_or(Anchor::CENTER);

        let (text_layout, text_bounds, bounds_policy) = {
            if let Some(text_layout) = world.entity(entity).get::<TextLayoutInfo>() {
                let text_bounds = world.entity(entity).get::<TextBounds>().copied();
                let bounds_policy = text_bounds.map_or(
                    TextWindowBoundsPolicy {
                        constrain_width: false,
                        constrain_height: false,
                    },
                    |bounds| TextWindowBoundsPolicy {
                        constrain_width: bounds.width.is_some(),
                        constrain_height: bounds.height.is_some(),
                    },
                );
                (text_layout.clone(), text_bounds, bounds_policy)
            } else {
                return;
            }
        };

        let content_size = if let Some(text_bounds) = text_bounds {
            Self::measured_text_size(&text_layout, &text_bounds)
        } else {
            text_layout.size
        }
        .max(Vec2::splat(1.0));
        let min_inner = (content_size + padding).max(Vec2::splat(1.0));
        let anchor_offset = get_anchor_offset(&anchor, content_size).extend(0.1);

        if let Some(mut draggable) = world.entity_mut(entity).get_mut::<Draggable>() {
            let edge_padding = 10.0;
            draggable.region = Some(DraggableRegion {
                region: Vec2::new(min_inner.x + edge_padding, header_height + edge_padding),
                offset: Vec2::new(
                    -anchor_offset.x,
                    (min_inner.y + header_height) * 0.5 - anchor_offset.y,
                ),
            });
        }

        let mut window_entity: Option<Entity> = None;
        world.commands().entity(entity).with_children(|parent| {
            window_entity = Some(
                parent
                    .spawn((
                        Window::new(
                            title,
                            HollowRectangle {
                                color,
                                dimensions: min_inner,
                                ..default()
                            },
                            header_height,
                            has_close_button,
                            Some(entity),
                        ),
                        WindowContentMetrics::from_min_inner(min_inner),
                        WindowOverflowPolicy::ConstrainToContent,
                        Transform::from_translation(-anchor_offset),
                    ))
                    .id(),
            );
        });

        if let Some(window_entity) = window_entity {
            world.commands().entity(entity).insert((
                WindowContentHost { window_entity },
                bounds_policy,
                TextWindowLayoutState {
                    base_content_size: content_size,
                    min_content_size: content_size,
                },
            ));
        }
    }
}

#[derive(Clone)]
pub struct TextContent {
    pub content: String,
    pub color: Color,
    pub size: f32,
    pub padding: Vec2,
    anchor: Anchor,
    bounds: Vec2,
}

impl Default for TextContent {
    fn default() -> Self {
        Self {
            content: String::from("Placeholder!"),
            color: Color::WHITE,
            size: 10.0,
            padding: Vec2::ZERO,
            anchor: Anchor::CENTER,
            bounds: Vec2::ONE,
        }
    }
}

impl TextContent {
    pub fn new(content: String, color: Color, size: f32) -> Self {
        Self {
            content,
            color,
            size,
            padding: Vec2::ZERO,
            anchor: Anchor::CENTER,
            bounds: Vec2::ONE,
        }
    }
}

#[derive(Clone, Component)]
#[component(on_insert = Cell::on_insert)]
#[require(Transform, Visibility)]
pub struct Cell {
    text: TextContent,
    border: BorderedRectangle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text: TextContent::default(),
            border: BorderedRectangle::default(),
        }
    }
}

#[derive(Component)]
struct CellTextNode;

impl Cell {
    pub fn new(text: TextContent) -> Self {
        Self { text, ..default() }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (text, border) = {
            if let Some(cell) = world.entity_mut(entity).get_mut::<Cell>() {
                (cell.text.clone(), cell.border)
            } else {
                return;
            }
        };

        let offset: Vec2 = get_anchor_offset(&text.anchor, text.bounds);

        let justify = justify_for_anchor(&text.anchor);

        world.commands().entity(entity).with_children(|parent| {
            parent.spawn((
                CellTextNode,
                Text2d(text.content),
                TextColor(text.color),
                text.anchor,
                TextLayout {
                    justify,
                    ..default()
                },
                TextFont {
                    font_size: scaled_font_size(text.size),
                    ..default()
                },
                TextBounds {
                    width: Some(text.bounds.x),
                    height: Some(text.bounds.y),
                    ..default()
                },
                Transform::from_translation(offset.extend(0.0)),
            ));

            parent.spawn(border);
        });
    }

    fn propagate_changes(
        changed_cells: Query<(Entity, &Cell), Changed<Cell>>,
        children_query: Query<&Children>,
        mut text_query: Query<
            (
                &mut TextBounds,
                &mut Transform,
                &mut Anchor,
                &mut TextLayout,
            ),
            With<CellTextNode>,
        >,
        mut border_query: Query<&mut BorderedRectangle>,
    ) {
        for (cell_entity, cell) in changed_cells.iter() {
            if let Ok(children) = children_query.get(cell_entity) {
                for child in children.iter() {
                    if let Ok((mut bounds, mut transform, mut anchor, mut layout)) =
                        text_query.get_mut(child)
                    {
                        let text_size = cell.text.bounds.max(Vec2::splat(1.0));
                        bounds.width = Some(text_size.x);
                        bounds.height = Some(text_size.y);
                        *anchor = cell.text.anchor;
                        layout.justify = justify_for_anchor(&cell.text.anchor);
                        transform.translation =
                            get_anchor_offset(&cell.text.anchor, text_size).extend(0.0);
                    }

                    if let Ok(mut border) = border_query.get_mut(child) {
                        *border = cell.border;
                    }
                }
            }
        }
    }

    fn recenter_text_vertically(
        mut text_query: Query<
            (&Anchor, &TextBounds, &TextLayoutInfo, &mut Transform),
            (
                With<CellTextNode>,
                Or<(
                    Changed<TextLayoutInfo>,
                    Changed<TextBounds>,
                    Changed<Anchor>,
                )>,
            ),
        >,
    ) {
        for (anchor, bounds, text_layout, mut transform) in text_query.iter_mut() {
            let width = bounds.width.unwrap_or(text_layout.size.x).max(1.0);
            let height = bounds.height.unwrap_or(text_layout.size.y).max(1.0);
            let offset = get_anchor_offset(anchor, Vec2::new(width, height));

            transform.translation.x = offset.x;

            let is_centered_vertical_anchor = matches!(
                anchor,
                &Anchor::CENTER_LEFT | &Anchor::CENTER | &Anchor::CENTER_RIGHT
            );
            if !is_centered_vertical_anchor {
                transform.translation.y = offset.y;
                continue;
            }

            let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
            for run in &text_layout.run_geometry {
                min_y = min_y.min(run.bounds.min.y);
                max_y = max_y.max(run.bounds.max.y);
            }

            let visual_center_y = if min_y.is_finite() && max_y.is_finite() {
                (min_y + max_y) * 0.5
            } else {
                height * 0.5
            };
            let center_correction = visual_center_y - height * 0.5;
            transform.translation.y = offset.y + center_correction;
        }
    }
}

#[derive(Clone, Component)]
#[component(on_insert = Column::on_insert)]
#[require(Transform, Visibility)]
pub struct Column {
    pub cells: Vec<Cell>,
    pub width: f32,
    pub padding: Vec2,
    pub anchor: Anchor,
    pub has_boundary: bool,
    rows: Vec<Row>,
}

impl Column {
    pub fn new(
        cells: Vec<Cell>,
        width: f32,
        padding: Vec2,
        anchor: Anchor,
        has_boundary: bool,
    ) -> Self {
        let num_cells = cells.len();
        Self {
            cells,
            width,
            padding,
            anchor,
            has_boundary,
            rows: vec![Row::default(); num_cells],
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (cells, padding, width, rows, anchor, has_boundary) = {
            if let Some(column) = world.entity_mut(entity).get_mut::<Column>() {
                (
                    column.cells.clone(),
                    column.padding,
                    column.width,
                    column.rows.clone(),
                    column.anchor,
                    column.has_boundary,
                )
            } else {
                return;
            }
        };

        world.commands().entity(entity).with_children(|parent| {
            let mut current_center_y = 0.0;
            let mut prev_height = 0.0;

            for (i, (mut cell, row)) in cells.into_iter().zip(rows.into_iter()).enumerate() {
                let height = row.height;
                if i == 0 {
                    // First row: its center is at -height/2.0 (y is downward)
                    current_center_y = -height / 2.0;
                } else {
                    // For subsequent rows, shift by half of the previous height and half of the current row's height.
                    current_center_y -= (prev_height / 2.0) + (height / 2.0);
                }

                cell.border.boundary.dimensions = Vec2::new(width, height);
                cell.border.boundary.sides = RectangleSides {
                    top: false,
                    bottom: false,
                    left: has_boundary,
                    right: false,
                };

                cell.text.bounds = (Vec2::new(width, height) - padding).max(Vec2::splat(1.0));
                cell.text.padding = padding;
                cell.text.anchor = anchor;

                parent.spawn((
                    cell.clone(),
                    Transform::from_translation(Vec3::new(0.0, current_center_y, 0.0)),
                ));

                prev_height = height;
            }
        });
    }

    fn propagate_changes(
        changed_columns: Query<(Entity, &Column), Changed<Column>>,
        children_query: Query<&Children>,
        mut cell_query: Query<(&mut Cell, &mut Transform)>,
    ) {
        for (column_entity, column) in changed_columns.iter() {
            let Ok(children) = children_query.get(column_entity) else {
                continue;
            };

            let mut current_center_y = 0.0;
            let mut prev_height = 0.0;
            let mut row_index = 0usize;

            for child in children.iter() {
                let Ok((mut cell, mut cell_transform)) = cell_query.get_mut(child) else {
                    continue;
                };

                let row_height = column
                    .rows
                    .get(row_index)
                    .map_or(10.0, |row| row.height.max(1.0));
                if row_index == 0 {
                    current_center_y = -row_height * 0.5;
                } else {
                    current_center_y -= (prev_height + row_height) * 0.5;
                }

                cell.border.boundary.dimensions = Vec2::new(column.width, row_height);
                cell.border.boundary.sides = RectangleSides {
                    top: false,
                    bottom: false,
                    left: column.has_boundary,
                    right: false,
                };
                cell.text.bounds =
                    (Vec2::new(column.width, row_height) - column.padding).max(Vec2::splat(1.0));
                cell.text.padding = column.padding;
                cell.text.anchor = column.anchor;

                if (cell_transform.translation.y - current_center_y).abs() > 0.001 {
                    cell_transform.translation.y = current_center_y;
                }

                prev_height = row_height;
                row_index += 1;
            }
        }
    }
}

#[derive(Clone)]
pub struct Row {
    pub height: f32,
}

impl Default for Row {
    fn default() -> Self {
        Self { height: 10.0 }
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
#[component(on_insert = Table::on_insert)]
pub struct Table {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

impl Table {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (columns, rows) = {
            if let Some(table) = world.entity_mut(entity).get_mut::<Table>() {
                (table.columns.clone(), table.rows.clone())
            } else {
                return;
            }
        };

        world.commands().entity(entity).with_children(|parent| {
            let mut current_center = 0.0;
            let mut prev_width = 0.0;

            for (i, mut column) in columns.into_iter().enumerate() {
                let width = column.width;

                if i == 0 {
                    // For the first column, place its center at half its width.
                    current_center = width / 2.0;
                } else {
                    // For subsequent columns, the center is moved by half of the previous width plus half of the current width.
                    current_center += (prev_width / 2.0) + (width / 2.0);
                }

                // Update the column with the row information.
                column.rows = rows.clone();

                // Spawn the column with its computed translation.
                let translation = Vec3::ZERO.with_x(current_center);
                parent.spawn((column, Transform::from_translation(translation)));

                prev_width = width;
            }
        });
    }

    fn propagate_changes(
        changed_tables: Query<(Entity, &Table), Changed<Table>>,
        children_query: Query<&Children>,
        mut columns_query: Query<(&mut Column, &mut Transform)>,
    ) {
        for (table_entity, table) in changed_tables.iter() {
            let Ok(children) = children_query.get(table_entity) else {
                continue;
            };

            let mut current_center_x = 0.0;
            let mut prev_width = 0.0;
            let mut column_index = 0usize;

            for child in children.iter() {
                let Ok((mut column, mut column_transform)) = columns_query.get_mut(child) else {
                    continue;
                };
                let Some(source_column) = table.columns.get(column_index) else {
                    break;
                };
                let width = source_column.width.max(1.0);

                if column_index == 0 {
                    current_center_x = width * 0.5;
                } else {
                    current_center_x += (prev_width + width) * 0.5;
                }

                column.width = width;
                column.rows = table.rows.clone();
                column.anchor = source_column.anchor;
                column.padding = source_column.padding;
                column.has_boundary = source_column.has_boundary;

                if (column_transform.translation.x - current_center_x).abs() > 0.001 {
                    column_transform.translation.x = current_center_x;
                }

                prev_width = width;
                column_index += 1;
            }
        }
    }
}
#[derive(Clone, Component)]
#[require(Transform, Visibility)]
#[component(on_insert = WindowedTable::on_insert)]
pub struct WindowedTable {
    pub title: Option<WindowTitle>,
    pub border_color: Color,
    pub header_height: f32,
    pub has_close_button: bool,
}

#[derive(Component, Clone)]
struct WindowedTableLayoutState {
    min_column_widths: Vec<f32>,
    min_row_heights: Vec<f32>,
}

impl WindowedTableLayoutState {
    fn min_inner(&self) -> Vec2 {
        let width = self
            .min_column_widths
            .iter()
            .fold(0.0, |acc, width| acc + width.max(1.0));
        let height = self
            .min_row_heights
            .iter()
            .fold(0.0, |acc, height| acc + height.max(1.0));
        Vec2::new(width, height)
    }
}

impl Default for WindowedTable {
    fn default() -> Self {
        Self {
            title: None,
            border_color: PRIMARY_COLOR,
            header_height: 20.0,
            has_close_button: true,
        }
    }
}

impl WindowedTable {
    fn distribute_sizes(min_sizes: &[f32], target_total: f32) -> Vec<f32> {
        if min_sizes.is_empty() {
            return vec![];
        }

        let min_total = min_sizes
            .iter()
            .fold(0.0, |acc, value| acc + value.max(1.0));
        let target_total = target_total.max(min_total);
        let extra = target_total - min_total;
        if extra <= 0.0 {
            return min_sizes.iter().map(|value| value.max(1.0)).collect();
        }

        min_sizes
            .iter()
            .map(|value| {
                let base = value.max(1.0);
                let weight = if min_total > 0.0 {
                    base / min_total
                } else {
                    1.0 / min_sizes.len() as f32
                };
                base + extra * weight
            })
            .collect()
    }

    fn apply_window_to_table(
        mut tables: Query<(
            &WindowedTable,
            &WindowedTableLayoutState,
            &WindowContentHost,
            Option<&mut Draggable>,
            &mut Table,
        )>,
        windows: Query<&Window>,
        changed_windows: Query<(), Changed<Window>>,
        mut window_transforms: Query<&mut Transform, With<Window>>,
    ) {
        for (windowed_table, layout_state, window_host, draggable, mut table) in tables.iter_mut() {
            if changed_windows.get(window_host.window_entity).is_err() {
                continue;
            }
            let Ok(window) = windows.get(window_host.window_entity) else {
                continue;
            };

            let inner_size = window.boundary.dimensions.max(Vec2::splat(1.0));
            let column_widths =
                Self::distribute_sizes(&layout_state.min_column_widths, inner_size.x);
            let row_heights = Self::distribute_sizes(&layout_state.min_row_heights, inner_size.y);

            for (column_index, column) in table.columns.iter_mut().enumerate() {
                if let Some(width) = column_widths.get(column_index) {
                    column.width = *width;
                }
            }
            for (row_index, row) in table.rows.iter_mut().enumerate() {
                if let Some(height) = row_heights.get(row_index) {
                    row.height = *height;
                }
            }

            if let Ok(mut window_transform) = window_transforms.get_mut(window_host.window_entity) {
                window_transform.translation.x = inner_size.x * 0.5;
                window_transform.translation.y = -inner_size.y * 0.5;
            }

            if let Some(mut draggable) = draggable {
                let edge_padding = 10.0;
                draggable.region = Some(DraggableRegion {
                    region: Vec2::new(
                        inner_size.x + edge_padding,
                        windowed_table.header_height + edge_padding,
                    ),
                    offset: Vec2::new(inner_size.x * 0.5, windowed_table.header_height * 0.5),
                });
            }
        }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let (columns, rows) = {
            if let Some(table) = world.entity_mut(entity).get_mut::<Table>() {
                (table.columns.clone(), table.rows.clone())
            } else {
                return;
            }
        };

        let min_column_widths: Vec<f32> =
            columns.iter().map(|column| column.width.max(1.0)).collect();
        let min_row_heights: Vec<f32> = rows.iter().map(|row| row.height.max(1.0)).collect();
        let layout_state = WindowedTableLayoutState {
            min_column_widths,
            min_row_heights,
        };
        let min_inner = layout_state.min_inner();

        let (title, header_height, color, has_close_button) = {
            if let Some(window) = world.entity(entity).get::<WindowedTable>() {
                (
                    window.title.clone(),
                    window.header_height,
                    window.border_color,
                    window.has_close_button,
                )
            } else {
                return;
            }
        };

        if let Some(mut draggable) = world.entity_mut(entity).get_mut::<Draggable>() {
            let edge_padding = 10.0;
            draggable.region = Some(DraggableRegion {
                region: Vec2::new(min_inner.x + edge_padding, header_height + edge_padding),
                offset: Vec2::new(min_inner.x * 0.5, header_height * 0.5),
            });
        }

        let mut window_entity: Option<Entity> = None;
        world.commands().entity(entity).with_children(|parent| {
            window_entity = Some(
                parent
                    .spawn((
                        Window::new(
                            title,
                            HollowRectangle {
                                color,
                                dimensions: min_inner,
                                ..default()
                            },
                            header_height,
                            has_close_button,
                            Some(entity),
                        ),
                        WindowContentMetrics::from_min_inner(min_inner),
                        WindowOverflowPolicy::ConstrainToContent,
                        Transform::from_xyz(min_inner.x * 0.5, -min_inner.y * 0.5, -0.1),
                    ))
                    .id(),
            );
        });

        if let Some(window_entity) = window_entity {
            world
                .commands()
                .entity(entity)
                .insert((WindowContentHost { window_entity }, layout_state));
        }
    }
}
