use bevy::prelude::*;

pub trait TextComponent: Component {
    fn new(text: impl Into<String>, translation: Vec3) -> impl Bundle;

    fn spawn(text: impl Into<String>, translation: Vec3, commands: &mut Commands) -> Entity {
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