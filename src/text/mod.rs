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
    interaction::{Clickable, KeyMapping, Pressable}, sprites::{BorderedBox, SpritePlugin, WindowBox, WindowTitle}, time::Dilation
};

pub struct TextPlugin;
impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app
		.add_systems(
			Update,
                (TextBox::update_box,
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
                ColorChangeEvent::Click(vec![CLICKED_BUTTON]),
                ColorChangeEvent::Hover(vec![HOVERED_BUTTON]),
            ]),
            Text2d::new(text),
        )
    }
}

pub struct TextBox(pub Vec2);

impl Default for TextBox{
    fn default() -> Self {
        Self(Vec2::new(20.0, 10.0))
    }
}

impl Component for TextBox {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let mut commands = world.commands();
                commands.entity(entity).with_children(|parent| {
                    parent.spawn(
                        BorderedBox::default()
                    );
                });
            }
        );
    }
}

impl TextBox {
    fn update_box(
        mut box_query: Query<(Entity, &TextBox, &TextLayoutInfo, &TextBounds, Option<&Anchor>), (With<TextBox>, Or<(Changed<TextLayoutInfo>, Changed<TextBounds>)>)>,
        mut background_query: Query<(&mut Transform, &mut BorderedBox)>,
        children_query: Query<&Children>,
    ) {
        for (entity, text_box, text_layout, text_bounds, anchor) in box_query.iter_mut() {

            // Get the text dimensions
            let text_width = text_bounds.width.unwrap_or(text_layout.size.x);
            let text_height = text_bounds.height.unwrap_or(text_layout.size.y);

            // Determine anchor, default to Center if None
            let anchor = anchor.unwrap_or(&Anchor::Center);
            let anchor_offset = match anchor {
                Anchor::TopLeft => Vec2::new(-text_width / 2.0, text_height / 2.0),
                Anchor::TopCenter => Vec2::new(0.0, text_height / 2.0),
                Anchor::TopRight => Vec2::new(text_width / 2.0, text_height / 2.0),
                Anchor::CenterLeft => Vec2::new(-text_width / 2.0, 0.0),
                Anchor::Center => Vec2::new(0.0, 0.0),
                Anchor::CenterRight => Vec2::new(text_width / 2.0, 0.0),
                Anchor::BottomLeft => Vec2::new(-text_width / 2.0, -text_height / 2.0),
                Anchor::BottomCenter => Vec2::new(0.0, -text_height / 2.0),
                Anchor::BottomRight => Vec2::new(text_width / 2.0, -text_height / 2.0),
                Anchor::Custom(offset) => *offset, // Use the custom anchor offset directly
            };

            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {

                    if let Ok((mut transform, mut bordered_box)) = background_query.get_mut(child) {

                        // Keep the z position slightly behind the text
                        transform.translation.z = -0.2;
                        transform.translation.x = -anchor_offset.x;
                        transform.translation.y = -anchor_offset.y;

                        bordered_box.0.x = text_width + text_box.0.x;
                        bordered_box.0.y = text_height + text_box.0.y;
                    }
                }
            }
            

        }
    }
}

#[derive(Clone)]
pub struct TextWindow{
    pub title : Option<WindowTitle>,
    pub header_height : f32,
    pub padding : Vec2
}

impl Default for TextWindow{
    fn default() -> Self {
        Self{
            title : None,
            header_height : 20.0,
            padding : Vec2::new(20.0, 10.0)
        }
    }
}

impl Component for TextWindow {
	const STORAGE_TYPE: StorageType = StorageType::Table;

	fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_insert(
            |mut world, entity, _component_id| {       

                let text_window: Option<TextWindow> = {
                    let entity_mut: EntityRef<'_> = world.entity(entity);
                    entity_mut.get::<TextWindow>()
                        .map(|bordered_box: &TextWindow| bordered_box.clone())
                };

                if let Some(text_window) = text_window {
                    let mut commands = world.commands();
                    commands.entity(entity).with_children(|parent| {
                        parent.spawn(
                            WindowBox{
                                size : Vec2::new(100.0, 100.0),
                                header_height  : text_window.header_height,
                                title : text_window.title.clone()
                            }
                        );
                    });
                }
            }
        );
    }
}

impl TextWindow {
    fn update_box(
        mut box_query: Query<(Entity, &TextWindow, &TextLayoutInfo, &TextBounds, Option<&Anchor>), (With<TextWindow>, Or<(Changed<TextLayoutInfo>, Changed<TextBounds>)>)>,
        mut background_query: Query<(&mut Transform, &mut WindowBox)>,
        children_query: Query<&Children>,
    ) {
        for (entity, text_box, text_layout, text_bounds, anchor) in box_query.iter_mut() {

            // Get the text dimensions
            let text_width = text_bounds.width.unwrap_or(text_layout.size.x);
            let text_height = text_bounds.height.unwrap_or(text_layout.size.y);

            // Determine anchor, default to Center if None
            let anchor = anchor.unwrap_or(&Anchor::Center);
            let anchor_offset = match anchor {
                Anchor::TopLeft => Vec2::new(-text_width / 2.0, text_height / 2.0),
                Anchor::TopCenter => Vec2::new(0.0, text_height / 2.0),
                Anchor::TopRight => Vec2::new(text_width / 2.0, text_height / 2.0),
                Anchor::CenterLeft => Vec2::new(-text_width / 2.0, 0.0),
                Anchor::Center => Vec2::new(0.0, 0.0),
                Anchor::CenterRight => Vec2::new(text_width / 2.0, 0.0),
                Anchor::BottomLeft => Vec2::new(-text_width / 2.0, -text_height / 2.0),
                Anchor::BottomCenter => Vec2::new(0.0, -text_height / 2.0),
                Anchor::BottomRight => Vec2::new(text_width / 2.0, -text_height / 2.0),
                Anchor::Custom(offset) => *offset, // Use the custom anchor offset directly
            };

            if let Ok(children) = children_query.get(entity) {
                for &child in children.iter() {

                    if let Ok((mut transform, mut windowed_box)) = background_query.get_mut(child) {

                        // Keep the z position slightly behind the text
                        transform.translation.z = -0.1;
                        transform.translation.x = -anchor_offset.x;
                        transform.translation.y = -anchor_offset.y;

                        windowed_box.size.x = text_width + text_box.padding.x;
                        windowed_box.size.y = text_height + text_box.padding.y;
                    }
                }
            }
        }
    }
}