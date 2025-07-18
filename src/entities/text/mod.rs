use serde::Deserialize;
use bevy::{
    ecs::{component::HookContext, world::DeferredWorld}, prelude::*, render::primitives::Aabb, sprite::Anchor, text::{
        TextBounds, TextLayoutInfo
    }
};

use crate::{
    systems::{
        colors::{
            ColorAnchor, 
            ColorChangeEvent, 
            ColorChangeOn, 
            CLICKED_BUTTON,
            HOVERED_BUTTON, 
            PRIMARY_COLOR
        }, 
        interaction::{
            Clickable, 
            Draggable, 
            DraggableRegion, 
            KeyMapping, 
            Pressable
        }, 
        time::Dilation
    },
    entities::sprites::{
        compound::{
            BorderedRectangle, 
            HollowRectangle, 
            RectangleSides
        }, 
        window::{
            Window, 
            WindowTitle
        },
        SpritePlugin
    }
};

pub struct TextPlugin;
impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			Update,
                TextWindow::propagate_changes
            );

        if !app.is_plugin_added::<SpritePlugin>() {
			app.add_plugins(SpritePlugin);
		}
    }
}

fn default_font() -> TextFont {
    TextFont{
        font_size : 12.0,
        ..default()
    }
}

fn default_font_color() -> TextColor {
    TextColor(PRIMARY_COLOR)
}

fn default_text_layout() -> TextLayout {
    TextLayout{
        justify: JustifyText::Center,
        ..default()
    }
}

fn default_nowrap_layout() -> TextLayout {
    TextLayout{
        justify: JustifyText::Center,
        linebreak : LineBreak::NoWrap,
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
    pub frames: Vec<String>
}

impl Default for TextFrames {
    fn default() -> Self {
        Self {
            frames: vec![]
        }
    }
}

impl TextFrames {
    pub fn new(frames: Vec<String>) -> Self {
        Self { frames }
    }

    pub fn load(input_string : &str) -> Result<Self, Box<dyn std::error::Error>> {
        match serde_json::from_str(input_string) {
            Ok(frames) => Ok(Self::new(frames)),
            Err(e) => {
                eprintln!("Failed to deserialize JSON from file: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
}


const CHAR_WIDTH: f32 = 7.0;
const LINE_HEIGHT: f32 = 16.0;

// ────────────────────────────────────────────────────────────────
//  CharacterSprite now carries figure dimensions so `offset()` works
// ────────────────────────────────────────────────────────────────
#[derive(Component)]
pub struct CharacterSprite {
    pub row : usize,
    pub col : usize,
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
pub struct GlyphString{
    pub text : String, 
    pub depth : f32    
}

impl GlyphString {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let glyph_string = world.entity(entity).get::<GlyphString>().unwrap();

        let text = glyph_string.text.clone();
        let depth = glyph_string.depth.clone();

        let lines: Vec<&str> = text.lines().collect();
        let lines_count      = lines.len();
        let max_cols         = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

        // 1. spawn each glyph as a child
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                world.commands().entity(entity).with_children(|p| {
                    p.spawn((
                        CharacterSprite { row, col, cols: max_cols, lines: lines_count },
                        TextSprite,
                        Text2d::new(ch.to_string()),
                        Anchor::BottomCenter,
                        Transform::from_translation(
                            CharacterSprite { row, col, cols: max_cols, lines: lines_count }.offset()
                        )
                    ));
                });
            }
        }

        // 2. add one parent-level AABB
        let half_x = max_cols  as f32 * CHAR_WIDTH  * 0.5;
        let half_y = lines_count as f32 * LINE_HEIGHT * 0.5;
        let half_z = depth * 0.5;
        let centre = Vec3::new(0.0, half_y, 0.0);

        world.commands().entity(entity).insert(Aabb {
            center:       centre.into(),
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
    pub fn new(current_frame : usize, time_seconds : f32) -> Self {
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
    let text_length = match text.lines().next() {
        Some(line) => line.len(),
        None => text.len(),
    };
    text_length as f32 * 7.92
}

pub fn get_text_height(text: &String) -> f32 {
    text.lines().count() as f32 * 12.0
}

#[derive(Component)]
#[require(
    Text2d = default_button_text(), 
    TextFont = default_button_font(), 
    TextColor = default_font_color(),
    TextBounds = default_button_bounds(),
    ColorAnchor
)]

pub struct TextButton;

fn default_button_text() -> Text2d {
    Text2d::new("Default Button Text")
}

fn default_button_font() -> TextFont {
    TextFont {
        font_size: 16.0,
        ..default()
    }
}

pub fn default_button_bounds() -> TextBounds {
    TextBounds {
        width: Some(1000.0),
        ..default()
    }
}

impl TextButton {
    pub fn new<T>(
        actions: Vec<T>,
        keys: Vec<KeyCode>,
        text: impl Into<String>
    ) -> impl Bundle
    where 
        T: Clone + Copy + std::fmt::Debug + std::fmt::Display + std::cmp::Eq + Send + Sync + 'static,  
    {
        (
            TextButton,
            TextLayout{
                justify: JustifyText::Center,
                ..default()
            },   
            Clickable::new(actions.clone()),
            Pressable::new(vec!(
                KeyMapping{
                    keys,
                    actions,
                    allow_repeated_activation : false  
                }
            )),
            ColorChangeOn::new(vec![
                ColorChangeEvent::Click(vec![CLICKED_BUTTON], None),
                ColorChangeEvent::Hover(vec![HOVERED_BUTTON], None),
            ]),
            Text2d::new(text.into()),
        )
    }
}


fn get_anchor_offset(anchor : &Anchor, dimensions : Vec2) -> Vec2 {
    let width = dimensions.x;
    let height = dimensions.y;

    match anchor {
        Anchor::TopLeft => Vec2::new(-width / 2.0, height / 2.0,),
        Anchor::TopCenter => Vec2::new(0.0, height / 2.0),
        Anchor::TopRight => Vec2::new(width / 2.0, height / 2.0),
        Anchor::CenterLeft => Vec2::new(-width / 2.0, 0.0),
        Anchor::Center => Vec2::new(0.0, 0.0),
        Anchor::CenterRight => Vec2::new(width / 2.0, 0.0),
        Anchor::BottomLeft => Vec2::new(-width / 2.0, -height / 2.0),
        Anchor::BottomCenter => Vec2::new(0.0, -height / 2.0),
        Anchor::BottomRight => Vec2::new(width / 2.0, -height / 2.0),
        Anchor::Custom(offset) => *offset
    }
}

#[derive(Clone, Component)]
#[require(Transform, Visibility)]
#[component(on_insert = TextWindow::on_insert)]
pub struct TextWindow{
    pub title : Option<WindowTitle>,
    pub border_color : Color,
    pub header_height : f32,
    pub padding : Vec2,
    pub has_close_button : bool
}

impl Default for TextWindow{
    fn default() -> Self {
        Self{
            title : None,
            border_color : PRIMARY_COLOR,
            header_height : 20.0,
            padding : Vec2::new(20.0, 10.0),
            has_close_button : true
        }
    }
}

impl TextWindow {
    fn propagate_changes(
        mut box_query: Query<(
            Entity, &TextWindow, &TextLayoutInfo, &TextBounds, Option<&Anchor>, Option<&mut Draggable>, &TextWindow
        ), Or<(Changed<TextLayoutInfo>, Changed<TextBounds>)>>,
        mut background_query: Query<(&mut Transform, &mut Window)>,
        children_query: Query<&Children>,
    ) {
        for (entity, text_window, text_layout, text_bounds, anchor, draggable, window) in box_query.iter_mut() {

            let width = text_bounds.width.unwrap_or(text_layout.size.x);
            let height = text_bounds.height.unwrap_or(text_layout.size.y);

            let dimensions = Vec2::new(width, height);

            let anchor = anchor.unwrap_or(&Anchor::Center);
            let anchor_offset = get_anchor_offset(anchor, dimensions).extend(0.1);

            if let Some(mut draggable) = draggable {
                let edge_padding = 10.0;
                draggable.region = Some(DraggableRegion{
                    region : Vec2::new(width + text_window.padding.x + edge_padding, window.header_height + edge_padding),
                    offset : Vec2::new(-anchor_offset.x, (height + text_window.padding.y + window.header_height) / 2.0 - anchor_offset.y)
                });
            }   

            if let Ok(children) = children_query.get(entity) {
                for child in children.iter() {

                    if let Ok((mut transform, mut rectangle)) = background_query.get_mut(child) {
                        transform.translation = -anchor_offset;
                        rectangle.boundary.dimensions = Vec2::new(width, height) + text_window.padding;
                    }
                }
            }
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let (title, header_height, color, has_close_button) = {
            if let Some(window) = world.entity(entity).get::<TextWindow>() {
                (window.title.clone(), window.header_height, window.border_color, window.has_close_button)
            } else {
                return;
            }
        };  

        let (text_layout, text_bounds) = {
            if let Some(text_layout) = world.entity(entity).get::<TextLayoutInfo>() {
                (text_layout.clone(), world.entity(entity).get::<TextBounds>())
            } else {
                return;
            }
        };

        let dimensions = if let Some(text_bounds) = text_bounds {
            Vec2::new(
                text_bounds.width.unwrap_or(text_layout.size.x),
                text_bounds.height.unwrap_or(text_layout.size.y)
            )
        } else {
            text_layout.size
        };

        world.commands().entity(entity).with_children(|parent| {
            parent.spawn(
                Window::new(
                    title,
                    HollowRectangle{
                        color,
                        dimensions,
                        ..default()
                    },
                    header_height,
                    has_close_button,
                    Some(entity)
                )
            );
        });
    }
}

#[derive(Clone)]
pub struct TextContent{
    pub content : String,
    pub color : Color,
    pub size : f32,
    pub padding : Vec2,
    anchor : Anchor,
    bounds : Vec2
}

impl Default for TextContent {
    fn default() -> Self {
        Self {
            content : String::from("Placeholder!"),
            color : Color::WHITE,
            size : 10.0,
            padding : Vec2::ZERO,
            anchor : Anchor::Center,
            bounds : Vec2::ONE
        }
    }
}

impl TextContent {
    pub fn new(content : String, color : Color, size : f32) -> Self {
        Self{ 
            content,
            color,
            size,
            padding : Vec2::ZERO,
            anchor : Anchor::Center,
            bounds : Vec2::ONE
        }
    }
}

#[derive(Clone, Component)]
#[component(on_insert = Cell::on_insert)]
#[require(Transform, Visibility)]
pub struct Cell{
    text : TextContent,
    border : BorderedRectangle
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text : TextContent::default(),
            border : BorderedRectangle::default()
        }
    }
}

impl Cell {
    pub fn new(
        text : TextContent,
    ) -> Self {
        Self {
            text,
            ..default()
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let (text, border) = {
            if let Some(cell) = world.entity_mut(entity).get_mut::<Cell>() {
                (cell.text.clone(), cell.border)
            } else {
                return;
            }
        };  

        let offset:Vec2 = get_anchor_offset(&text.anchor, text.bounds); 

        let justify = match text.anchor {
            Anchor::TopLeft | Anchor::CenterLeft | Anchor::BottomLeft => JustifyText::Left,
            Anchor::TopCenter | Anchor::Center | Anchor::BottomCenter => JustifyText::Center,
            Anchor::TopRight | Anchor::CenterRight | Anchor::BottomRight => JustifyText::Right,
            _ => JustifyText::Center
        };

        world.commands().entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d(text.content),
                TextColor(text.color),
                text.anchor,
                TextLayout{
                    justify,
                    ..default()
                },
                TextFont{
                    font_size : text.size,
                    ..default()
                },
                TextBounds{
                    width : Some(text.bounds.x),
                    height : Some(text.bounds.y),
                    ..default()
                },
                Transform::from_translation(offset.extend(0.0))
            ));
            
            parent.spawn(border);
        });
    }
}

#[derive(Clone, Component)]
#[component(on_insert = Column::on_insert)]
#[require(Transform, Visibility)]
pub struct Column{
    pub cells : Vec<Cell>,
    pub width : f32,
    pub padding : Vec2,
    pub anchor : Anchor,
    pub has_boundary : bool,
    rows : Vec<Row>,
}

impl Column {
    pub fn new(cells : Vec<Cell>, width : f32, padding : Vec2, anchor : Anchor, has_boundary : bool) -> Self{
        
        let num_cells = cells.len();
        Self {
            cells,
            width,
            padding,
            anchor,
            has_boundary,
            rows : vec![Row::default(); num_cells]
        }
    }

    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let (cells, padding, width, rows, anchor, has_boundary) = {
            if let Some(column) = world.entity_mut(entity).get_mut::<Column>() {
                (column.cells.clone(), column.padding, column.width, column.rows.clone(), column.anchor, column.has_boundary)
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
                cell.border.boundary.sides = RectangleSides{
                    top : false, 
                    bottom : false,
                    left : has_boundary,
                    right : false
                };

                cell.text.bounds = Vec2::new(width, height) - padding;
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
}

#[derive(Clone)]
pub struct Row{
    pub height : f32
}

impl Default for Row {
    fn default() -> Self {
        Self{height : 10.0}
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
#[component(on_insert = Table::on_insert)]
pub struct Table{
    pub columns : Vec<Column>,
    pub rows : Vec<Row>
}

impl Table {
    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
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
                parent.spawn((
                    column,
                    Transform::from_translation(translation),
                ));

                prev_width = width;
            }
        });
    }
}
#[derive(Clone, Component)]
#[require(Transform, Visibility)]
#[component(on_insert = WindowedTable::on_insert)]
pub struct WindowedTable{
    pub title : Option<WindowTitle>,
    pub border_color : Color,
    pub header_height : f32,
    pub has_close_button : bool
}

impl Default for WindowedTable{
    fn default() -> Self {
        Self{
            title : None,
            border_color : PRIMARY_COLOR,
            header_height : 20.0,
            has_close_button : true
        }
    }
}

impl WindowedTable {
    fn on_insert(
        mut world : DeferredWorld,
        HookContext{entity, ..} : HookContext
    ) {
        let (columns, rows) = {
            if let Some(table) = world.entity_mut(entity).get_mut::<Table>() {
                (table.columns.clone(), table.rows.clone())
            } else {
                return;
            }
        };  

        let width = columns.iter().fold(0.0, |acc, column| acc + column.width);
        let height = rows.iter().fold(0.0, |acc, row| acc + row.height);

        let (title, header_height, color, has_close_button) = {
            if let Some(window) = world.entity(entity).get::<WindowedTable>() {
                (window.title.clone(), window.header_height, window.border_color, window.has_close_button)
            } else {
                return;
            }
        };  

        if let Some(mut draggable) = world.entity_mut(entity).get_mut::<Draggable>() {
            let edge_padding = 10.0;
            draggable.region = Some(DraggableRegion{
                region : Vec2::new(width, header_height + edge_padding),
                offset : Vec2::new(width/2.0, -height/2.0 + (height + header_height)/2.0)
            });
        }   

        world.commands().entity(entity).with_children(|parent| {
            parent.spawn((
                Window::new(
                    title,
                    HollowRectangle{
                        color,
                        dimensions : Vec2::new(width, height),
                        ..default()
                    },
                    header_height,
                    has_close_button,
                    Some(entity)
                ),
                Transform::from_xyz(width/2.0, -height/2.0, -0.1)
            ));
        });
    }
}