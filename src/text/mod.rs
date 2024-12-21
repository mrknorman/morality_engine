use std::{
    fs::File,
    io::BufReader,
    path::Path
};
use serde::Deserialize;

use bevy::{prelude::*, text::TextBounds};

use crate::{
    colors::{ColorAnchor, ColorChangeEvent, ColorChangeOn, CLICKED_BUTTON, HOVERED_BUTTON, PRIMARY_COLOR}, interaction::{Clickable, InputAction, Pressable}
};

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

#[derive(Component)]
#[require(TextFont(default_font), TextColor(default_font_color), TextLayout(default_nowrap_layout))]
pub struct TextSprite;

impl Default for TextSprite {
    fn default() -> Self {
        TextSprite
    }
}

#[derive(Component)]
#[require(TextFont(default_font), TextColor(default_font_color), TextLayout(default_nowrap_layout))]
pub struct TextTitle;

impl Default for TextTitle {
    fn default() -> Self {
        TextTitle
    }
}

#[derive(Component, Deserialize)]
pub struct TextFrames {
    pub frames: Vec<String>
}

impl Default for TextFrames {
    fn default() -> Self {
        TextFrames{
            frames : vec![]
        }
    }
}

impl TextFrames {
    pub fn new(frames: Vec<String>) -> Self {
        Self {
            frames 
        }
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
pub struct Animated{
    pub current_frame: usize,
    pub timer: Timer
}

impl Default for Animated {
    fn default() -> Self {
        Animated {
            current_frame: 0,
            timer: Timer::from_seconds(
                0.1, 
                TimerMode::Repeating
            )
        }
    }
}

impl Animated {
    pub fn animate_text(
        time: Res<Time>,
        mut query: Query<(&mut Animated, &TextFrames, &mut Text2d)>,
    ) {
        for (
                mut animation, frames, mut text
            ) in query.iter_mut() {
                animation.timer.tick(time.delta());
            if animation.timer.just_finished() {
                animation.current_frame = 
                    (animation.current_frame + 1) % frames.frames.len();
                text.0 = frames.frames[animation.current_frame].clone();
            }
        }
    }
}

pub fn get_text_width(text : &String) -> f32 {
    let text_length = match text.lines().next() {
        Some(line) => line.len(),
        None => text.len()
    };
    text_length as f32 * 7.92
}

pub fn get_text_height(text : &String) -> f32 {
    text.lines().count() as f32 * 12.0
}

#[derive(Component)]
#[require(Text2d(default_button_text), TextFont(default_button_font), TextColor(default_font_color), TextBounds(default_button_bounds), ColorAnchor)]
pub struct TextButton;

fn default_button_text() -> Text2d {
    Text2d::new("Default Button Text")
}

fn default_button_font() -> TextFont {
    TextFont{
        font_size : 16.0,
        ..default()
    }
}


pub fn default_button_bounds() -> TextBounds {
    TextBounds{
        width : Some(1000.0),
        ..default()
    }
}

impl TextButton {
    pub fn new( 
        actions : Vec<InputAction>,
        keys : Vec<KeyCode>,
        text: impl Into<String>
    ) -> (TextButton, Clickable, Pressable, ColorChangeOn, Text2d) {
        
        let text = text.into();
        (
            TextButton, 
            Clickable::new(actions.clone()),
            Pressable::new(
                keys,
                actions
            ),
            ColorChangeOn::new(vec![
                ColorChangeEvent::Click(vec![CLICKED_BUTTON]),
                ColorChangeEvent::Hover(vec![HOVERED_BUTTON]),
            ]),
            Text2d::new(text),
        )
    }
}