use bevy::prelude::*;

use crate::{
    interaction::{InputAction, Clickable, Pressable},
    audio::{TransientAudioPallet, TransientAudio}
};

pub trait TextComponent: Component {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle;
}

fn create_text_bundle(text: impl Into<String>, translation: Vec3, font_size: f32) -> Text2dBundle {
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

#[derive(Component)]
pub struct TextRaw;
impl TextComponent for TextRaw {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle {
        (TextRaw, create_text_bundle(text, translation, 12.0))
    }
}

#[derive(Component)]
pub struct TextSprite;
impl TextComponent for TextSprite {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle {
        (TextSprite, create_text_bundle(text, translation, 12.0))
    }
}

#[derive(Component)]
pub struct TextTitle;
impl TextComponent for TextTitle {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle {
        (TextTitle, create_text_bundle(text, translation, 12.0))
    }
}

#[derive(Component)]
pub struct AnimatedTextSprite {
    pub frames: Vec<String>,
    pub current_frame: usize,
    pub timer: Timer,
}

impl AnimatedTextSprite {

    pub fn from_vec(
            frames: Vec<String>, frame_time_seconds : f32, translation: Vec3
        ) -> impl Bundle {

        (
            TextSprite,
            AnimatedTextSprite {
                frames : frames.clone(),
                current_frame: 0,
                timer: Timer::from_seconds(
                    frame_time_seconds, 
                    TimerMode::Repeating
                )
            },
            create_text_bundle(frames[0].clone(), translation, 12.0),
        )
    }

    pub fn animate_text_sprites(
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
impl TextButton {
    pub fn new(
            commands: &mut Commands,
            asset_server: &Res<AssetServer>,
		    parent_entity: Option<Entity>,
            actions : Vec<InputAction>,
            keys : Vec<KeyCode>,
            text: impl Into<String>, 
            translation: Vec3
        ) {

        let bundle = (
            TextButton, 
            Clickable::new(actions.clone(), Vec2::new(285.0,20.0)),
            create_text_bundle(text, translation, 16.0)
        );

        let button_entity = if let Some(parent) = parent_entity {

            let mut button_entity : Option<Entity> = None;
            commands.entity(parent).with_children(|parent| {
                button_entity = Some(parent.spawn(bundle).id());
            });

            button_entity.unwrap()
        } else {
            commands.spawn(bundle).id()
        };

        if keys.len() > 0 {
            commands.entity(button_entity).insert(Pressable::new(
                keys,
                actions
            ));
        }

        TransientAudioPallet::insert(
            vec![(
                "click".to_string(),
                TransientAudio::new(
                    "sounds/mech_click.ogg", 
                    asset_server, 
                    0.1, 
                    true,
                    1.0
                ),
            )],
            &mut commands.entity(button_entity)
        );
    }
}

