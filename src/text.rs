use bevy::prelude::*;

pub trait TextComponent: Component {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle;

    fn spawn(
            text: impl Into<String>, translation: Vec3, commands: &mut Commands
        ) -> Entity {

        commands.spawn(Self::new(text, translation)).id()
    }
}

fn create_text_bundle(text: impl Into<String>, translation: Vec3) -> Text2dBundle {
    Text2dBundle {
        text: Text {
            sections: vec![TextSection::new(
                text.into(),
                TextStyle {
                    font_size: 12.0,
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
pub struct TextSprite;
impl TextComponent for TextSprite {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle {
        (TextSprite, create_text_bundle(text, translation))
    }
}

#[derive(Component)]
pub struct TextTitle;
impl TextComponent for TextTitle {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle {
        (TextTitle, create_text_bundle(text, translation))
    }
}

#[derive(Component)]
pub struct AnimatedTextSprite {
    pub frames: Vec<String>,
    pub current_frame: usize,
    pub frame_time_seconds: f32,
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
                frame_time_seconds,
                timer: Timer::from_seconds(
                    frame_time_seconds, 
                    TimerMode::Repeating
                )
            },
            create_text_bundle(frames[0].clone(), translation),
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


