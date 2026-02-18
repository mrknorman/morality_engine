mod merlin;

use std::time::Duration;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use enum_map::{enum_map, Enum};

use crate::{
    entities::text::{get_text_height, get_text_width, scaled_text_units, TextTitle},
    systems::{
        audio::{TransientAudio, TransientAudioPallet},
        colors::{
            ColorAnchor, ColorChangeEvent, ColorChangeOn, Flicker, OPTION_1_COLOR, OPTION_2_COLOR,
        },
        interaction::{ActionPallet, Clickable, InputAction},
        motion::{Bounce, Wobble},
    },
};

#[derive(Default, States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AsciiSystemsActive {
    #[default]
    False,
    True,
}

pub struct AsciiPlugin;
impl Plugin for AsciiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AsciiSystemsActive>()
            .add_systems(Update, activate_systems);
    }
}

fn activate_systems(
    mut ascii_state: ResMut<NextState<AsciiSystemsActive>>,
    ascii_query: Query<&AsciiString>,
) {
    ascii_state.set(if ascii_query.is_empty() {
        AsciiSystemsActive::False
    } else {
        AsciiSystemsActive::True
    });
}

#[derive(Component, Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEmotion {
    Happy,
    Afraid,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiActions {
    BounceOnClick,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsciiSounds {
    Bounce,
}

#[derive(Clone, Component)]
#[component(on_insert = AsciiString::on_insert)]
#[require(Transform, Visibility)]
pub struct AsciiString(pub String);

#[derive(Component, Clone)]
#[require(TextTitle)]
pub struct AsciiChar;

impl AsciiString {
    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        // If this component is reinserted on the same entity, clear old glyph children
        // so large-font titles cannot duplicate and overlap.
        world.commands().entity(entity).despawn_related::<Children>();

        // Get the text and components early to avoid borrow conflicts
        let (text, color, beep, emotion) = {
            let entity_ref = world.entity(entity);
            let text = match entity_ref.get::<AsciiString>() {
                Some(text) => text.0.clone(),
                None => return,
            };

            let color = entity_ref.get::<TextColor>().cloned();
            let emotion = entity_ref.get::<TextEmotion>().cloned();

            let asset_server = world.get_resource::<AssetServer>().unwrap();
            let beep: Handle<AudioSource> = asset_server.load("./audio/effects/beep.ogg");

            (text, color, beep, emotion)
        };

        let mut placements: Vec<(String, Vec3)> = Vec::new();
        let mut bounds_min = Vec2::splat(f32::INFINITY);
        let mut bounds_max = Vec2::splat(f32::NEG_INFINITY);
        let mut line_cursor_y = 0.0;

        for line in text.lines() {
            let line_glyphs = merlin::get_vector(line.to_string());
            if line_glyphs.is_empty() {
                continue;
            }

            let line_height = get_text_height(&line_glyphs[0]) + scaled_text_units(15.0);
            let mut line_cursor_x = 0.0;

            for ascii_char in line_glyphs {
                let char_width = get_text_width(&ascii_char);
                let char_height = get_text_height(&ascii_char);
                let translation = Vec3::new(line_cursor_x, line_cursor_y, 0.0);
                let half_extent = Vec2::new(char_width * 0.5, char_height * 0.5);
                let min = Vec2::new(translation.x - half_extent.x, translation.y - half_extent.y);
                let max = Vec2::new(translation.x + half_extent.x, translation.y + half_extent.y);
                bounds_min = bounds_min.min(min);
                bounds_max = bounds_max.max(max);
                placements.push((ascii_char, translation));

                line_cursor_x += char_width - scaled_text_units(15.0);
            }

            line_cursor_y -= line_height;
        }

        if placements.is_empty() {
            return;
        }

        let center = (bounds_min + bounds_max) * 0.5;
        let mut commands = world.commands();
        commands.entity(entity).with_children(|parent| {
            for (ascii_char, translation) in placements {
                let centered_translation =
                    Vec3::new(translation.x - center.x, translation.y - center.y, translation.z);

                // Spawn entity with base components
                let mut entity_cmd = parent.spawn((
                    AsciiChar,
                    TextTitle,
                    TextLayout {
                        justify: Justify::Left,
                        linebreak: LineBreak::NoWrap,
                        ..default()
                    },
                    ColorAnchor::default(),
                    Text2d::new(ascii_char),
                    Transform::from_translation(centered_translation),
                ));

                // Add color if available
                if let Some(color) = &color {
                    entity_cmd.insert(color.clone());
                }

                // Add emotion-specific components
                match &emotion {
                    Some(TextEmotion::Happy) => {
                        entity_cmd.insert((
                            Bounce::default(),
                            Clickable::new(vec![AsciiActions::BounceOnClick]),
                            ActionPallet::<AsciiActions, AsciiSounds>(enum_map! {
                                AsciiActions::BounceOnClick => vec![
                                    InputAction::Bounce,
                                    InputAction::PlaySound(AsciiSounds::Bounce)
                                ]
                            }),
                            TransientAudioPallet::new(vec![(
                                AsciiSounds::Bounce,
                                vec![TransientAudio::new(beep.clone(), 0.1, true, 1.0, true)],
                            )]),
                            ColorChangeOn::new(vec![ColorChangeEvent::Bounce(vec![
                                OPTION_1_COLOR,
                                OPTION_2_COLOR,
                            ])]),
                        ));
                    }
                    Some(TextEmotion::Afraid) => {
                        entity_cmd.insert((
                            Wobble::default(),
                            ColorChangeOn::new(vec![ColorChangeEvent::Flicker(
                                vec![OPTION_1_COLOR, OPTION_2_COLOR],
                                Flicker::new(
                                    Duration::from_secs_f32(0.1),
                                    Duration::from_secs_f32(1.0),
                                    Duration::from_secs_f32(2.0),
                                    Duration::from_secs_f32(3.0),
                                    Duration::from_secs_f32(0.5),
                                ),
                            )]),
                        ));
                    }
                    None => {}
                }
            }
        });
    }
}
