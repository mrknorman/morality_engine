use std::{
    fs::File, io::BufReader, path::Path
};
use serde::Deserialize;

use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*, sprite::Anchor, text::{TextBounds, TextLayoutInfo}};

use crate::{
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
    sprites::{
        compound::{BorderedRectangle, HollowRectangle}, window::{Window, WindowTitle}, SpritePlugin
    }, 
    time::Dilation
};

pub struct TextPlugin;
impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			Update,
                (
                TextBox::update_box,
                TextWindow::update_box)
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
#[require(TextFont(default_font), TextColor(default_font_color), TextLayout(default_text_layout))]
pub struct TextRaw;

impl Default for TextRaw {
    fn default() -> Self {
        Self
    }
}

#[derive(Component)]
#[require(TextFont(default_font), TextColor(default_font_color), TextLayout(default_nowrap_layout))]
pub struct TextSprite;

impl Default for TextSprite {
    fn default() -> Self {
        Self
    }
}

#[derive(Component)]
#[require(TextFont(default_font), TextColor(default_font_color), TextLayout(default_nowrap_layout))]
pub struct TextTitle;

impl Default for TextTitle {
    fn default() -> Self {
        Self
    }
}

#[derive(Component, Deserialize)]
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

    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        match File::open(&file_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Ok(frames) => Ok(Self::new(frames)),
                    Err(e) => {
                        eprintln!("Failed to deserialize JSON from file: {:?}", e);
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to open file: {:?}", e);
                Err(Box::new(e))
            }
        }
    }
}

#[derive(Component)]
#[require(TextSprite, Text2d, TextFrames)]
pub struct Animated {
    pub current_frame: usize,
    pub timer: Timer,
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
#[require(Text2d(default_button_text), TextFont(default_button_font), TextColor(default_font_color), TextBounds(default_button_bounds), ColorAnchor)]
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
    ) -> (TextButton, Clickable<T>, Pressable<T>, ColorChangeOn, Text2d)
    where 
        T: Clone + Copy + std::fmt::Debug + std::fmt::Display + std::cmp::Eq + Send + Sync,  
    {
        let text = text.into();
        (
            TextButton,
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
            Text2d::new(text),
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


pub struct TextBox{
    pub padding : Vec2
}
impl Default for TextBox{
    fn default() -> Self {
        Self{
            padding : Vec2::new(20.0, 10.0)
        }
    }
}

impl Component for TextBox {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       
                world.commands().entity(entity).with_children(|parent| {
                    parent.spawn(
                        BorderedRectangle::default()
                    );
                });
            }
        );
    }
}

impl TextBox {
    fn update_box(
        mut box_query: Query<(Entity, &TextBox, &TextLayoutInfo, &TextBounds, Option<&Anchor>), (With<TextBox>, Or<(Changed<TextLayoutInfo>, Changed<TextBounds>)>)>,
        mut background_query: Query<(&mut Transform, &mut BorderedRectangle)>,
        children_query: Query<&Children>,
    ) {
        for (entity, text_box, text_layout, text_bounds, anchor) in box_query.iter_mut() {

            let dimensions = Vec2::new(
                text_bounds.width.unwrap_or(text_layout.size.x), 
                text_bounds.height.unwrap_or(text_layout.size.y)
            );

            let anchor = anchor.unwrap_or(&Anchor::Center);
            let anchor_offset = get_anchor_offset(anchor, dimensions).extend(0.1);
            
            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {
                    if let Ok((mut transform, mut rectangle)) = background_query.get_mut(child) {
                        transform.translation = -anchor_offset;
                        rectangle.boundary.dimensions = dimensions + text_box.padding;
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
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

impl Component for TextWindow {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let (title, header_height, color, has_close_button) = {
                    if let Some(window) = world.entity(entity).get::<TextWindow>() {
                        (window.title.clone(), window.header_height, window.border_color, window.has_close_button)
                    } else {
                        return;
                    }
                };  

                world.commands().entity(entity).with_children(|parent| {
                    parent.spawn(
                        Window::new(
                            title,
                            HollowRectangle{
                                color,
                                ..default()
                            },
                            header_height,
                            has_close_button,
                            Some(entity)
                        )
                    );
                });
            }
        );
    }
}

impl TextWindow {
    fn update_box(
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
                for &child in children.iter() {

                    if let Ok((mut transform, mut rectangle)) = background_query.get_mut(child) {
                        transform.translation = -anchor_offset;
                        rectangle.boundary.dimensions = Vec2::new(width, height) + text_window.padding;
                    }
                }
            }
        }
    }
}