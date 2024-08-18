use bevy::prelude::*;

use crate::{
    interaction::{InputAction, Clickable, Pressable},
    audio::{TransientAudioPallet, TransientAudio}
};

fn create_text_bundle(
        text: impl Into<String>, 
        translation: Vec3, 
        font_size: f32
    ) -> Text2dBundle {
    
    Text2dBundle {
        text: Text {
            sections: vec![TextSection::new(
                text.into(),
                TextStyle {
                    font_size,
                    ..default()
                },
            )],
            justify: JustifyText::Center,
            ..default()
        },
        transform: Transform::from_translation(translation),
        ..default()
    }
}
// Existing components
#[derive(Component, Clone)]
pub struct TextRaw;

#[derive(Component, Clone)]
pub struct TextSprite;

#[derive(Component, Clone)]
pub struct TextTitle;

// Bundle struct for TextRaw
#[derive(Bundle, Clone)]
pub struct TextRawBundle {
    marker: TextRaw,
    text: Text2dBundle,
}

impl TextRawBundle {
    pub fn new(text: impl Into<String>, translation: Vec3) -> Self {
        Self {
            marker: TextRaw,
            text: create_text_bundle(text, translation, 12.0),
        }
    }
}

// Bundle struct for TextSprite
#[derive(Bundle, Clone)]
pub struct TextSpriteBundle {
    marker: TextSprite,
    text: Text2dBundle,
}

impl TextSpriteBundle {
    pub fn new(text: impl Into<String>, translation: Vec3) -> Self {
        Self {
            marker: TextSprite,
            text: create_text_bundle(text, translation, 12.0),
        }
    }
}

// Bundle struct for TextTitle
#[derive(Bundle, Clone)]
pub struct TextTitleBundle {
    marker: TextTitle,
    text: Text2dBundle,
}

impl TextTitleBundle {
    pub fn new(text: impl Into<String>, translation: Vec3) -> Self {
        Self {
            marker: TextTitle,
            text: create_text_bundle(text, translation, 12.0),
        }
    }
}

#[derive(Component, Clone)]
pub struct AnimatedTextSprite {
    pub frames: Vec<String>,
    pub current_frame: usize,
    pub timer: Timer
}

#[derive(Bundle, Clone)]
pub struct AnimatedTextSpriteBundle {
    text_sprite_bundle: TextSpriteBundle,
    animation : AnimatedTextSprite
}

impl AnimatedTextSpriteBundle {
    pub fn from_vec(
        frames: Vec<String>, 
        frame_time_seconds : f32, 
        translation : Vec3
    ) -> Self {

        Self {
            text_sprite_bundle : TextSpriteBundle::new(frames[0].clone(), translation),
            animation : AnimatedTextSprite {
                frames : frames.clone(),
                current_frame: 0,
                timer: Timer::from_seconds(
                    frame_time_seconds, 
                    TimerMode::Repeating
                )
            }
        }
    }

    pub fn animate(
        time: Res<Time>,
        mut query: Query<(&mut AnimatedTextSprite, &mut Text)>,
    ) {
        for (
                mut animated_sprite, mut text
            ) in query.iter_mut() {
                
            animated_sprite.timer.tick(time.delta());
            if animated_sprite.timer.just_finished() {
                animated_sprite.current_frame = 
                    (animated_sprite.current_frame + 1) % animated_sprite.frames.len();
                text.sections[0].value = 
                    animated_sprite.frames[animated_sprite.current_frame].clone();
            }
        }
    }

}

#[derive(Component)]
pub struct TextButton;

#[derive(Bundle)]
pub struct TextButtonBundle {
    marker : TextButton,
    clickable : Clickable,
    text : Text2dBundle,
    audio : TransientAudioPallet,
    pressable : Pressable
}

impl TextButtonBundle {

    pub fn new( 
        asset_server: &Res<AssetServer>,
        actions : Vec<InputAction>,
        keys : Vec<KeyCode>,
        text: impl Into<String>, 
        translation: Vec3
    ) -> Self {
        
        Self {
            marker : TextButton, 
            clickable : Clickable::new(actions.clone(), Vec2::new(285.0,20.0)),
            text : create_text_bundle(text, translation, 16.0),
            audio : TransientAudioPallet::new(
                vec![(
                    "click".to_string(),
                    TransientAudio::new(
                        "sounds/mech_click.ogg", 
                        asset_server, 
                        0.1, 
                        true,
                        1.0
                    ),
                )]
            ),
            pressable : Pressable::new(
                keys,
                actions
            )
        }
    }
}