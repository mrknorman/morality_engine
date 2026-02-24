use std::time::Duration;

use bevy::{audio::Volume, prelude::*, sprite::Anchor, text::TextBounds};
use enum_map::{enum_map, Enum};

use crate::{
    data::{
        states::{DilemmaPhase, GameState, StateVector},
        stats::DilemmaStats,
    },
    entities::{
        text::{scaled_font_size, TextTitle, TextWindow},
        track::Track,
        train::Train,
    },
    scenes::dilemma::{
        dilemma::{CurrentDilemmaStageIndex, DilemmaStage, DilemmaTimer},
        lever::{Lever, LeverRoot, LeverState, LEVER_BASE},
        DilemmaSounds,
    },
    style::common_ui::CenterLever,
    startup::render::ScreenShakeState,
    systems::{
        audio::{
            continuous_audio, ContinuousAudio, ContinuousAudioPallet, TransientAudio,
            TransientAudioPallet,
        },
        backgrounds::Background,
        colors::{
            option_color, ColorAnchor, ColorChangeEvent, ColorChangeOn, ColorTranslation,
            DANGER_COLOR, PRIMARY_COLOR,
        },
        interaction::{
            ActionPallet, Clickable, ClickablePong, Draggable, InputAction, InteractionState,
            KeyMapping, Pressable,
        },
        motion::PointToPointTranslation,
        ui::window::UiWindowTitle,
    },
};

pub struct DilemmaDecisionPlugin;
impl Plugin for DilemmaDecisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(DilemmaPhase::Decision),
            DecisionScene::setup.run_if(in_state(GameState::Dilemma)),
        )
        .add_systems(
            Update,
            DecisionScene::update_stats.run_if(resource_changed::<Lever>),
        )
        .add_systems(
            Update,
            DecisionScene::sync_pong_state_to_lever.run_if(resource_changed::<Lever>),
        )
        .add_systems(
            Update,
            DecisionScene::update_screen_shake
                .run_if(in_state(GameState::Dilemma))
                .run_if(in_state(DilemmaPhase::Decision)),
        )
        .add_systems(
            OnExit(DilemmaPhase::Decision),
            (
                DecisionScene::cleanup,
                DecisionScene::finalize_stats,
                DecisionScene::clear_screen_shake,
            ),
        );
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionActions {
    LockDecision,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeverActions {
    SelectOption1,
    SelectOption2,
    SelectOption3,
    SelectOption4,
    SelectOption5,
    SelectOption6,
    SelectOption7,
    SelectOption8,
    SelectOption9,
}

#[derive(Component)]
#[require(Transform, Visibility)]
struct DecisionScene;

impl DecisionScene {
    const TIMER_TRANSLATION: Vec3 = Vec3::new(0.0, -100.0, 1.0);
    const LEVER_CLICK_REGION: Vec2 = Vec2::new(240.0, 360.0);

    // Decision option windows use deterministic two-column placement.
    const OPTION_WINDOW_COLUMNS: usize = 2;
    const OPTION_WINDOW_WIDTH: f32 = 400.0;
    const OPTION_WINDOW_MIN_HEIGHT_ESTIMATE: f32 = 180.0;
    const OPTION_WINDOW_LEFT_X: f32 = -600.0;
    const OPTION_WINDOW_RIGHT_X: f32 = 200.0;
    const OPTION_WINDOW_TOP_Y: f32 = -200.0;
    const OPTION_WINDOW_ROW_SPACING: f32 = -180.0;
    const OPTION_WINDOW_Z: f32 = 2.0;
    const OPTION_WINDOW_LINE_HEIGHT_ESTIMATE: f32 = 16.0;
    const OPTION_WINDOW_HEADER_HEIGHT: f32 = 20.0;
    const OPTION_WINDOW_VERTICAL_PADDING: f32 = 10.0;
    const OPTION_WINDOW_CHARS_PER_LINE_ESTIMATE: usize = 42;

    fn lever_action_for_option(option_index: usize) -> Option<LeverActions> {
        match option_index {
            0 => Some(LeverActions::SelectOption1),
            1 => Some(LeverActions::SelectOption2),
            2 => Some(LeverActions::SelectOption3),
            3 => Some(LeverActions::SelectOption4),
            4 => Some(LeverActions::SelectOption5),
            5 => Some(LeverActions::SelectOption6),
            6 => Some(LeverActions::SelectOption7),
            7 => Some(LeverActions::SelectOption8),
            8 => Some(LeverActions::SelectOption9),
            _ => None,
        }
    }

    fn digit_key_for_option(option_index: usize) -> Option<KeyCode> {
        match option_index {
            0 => Some(KeyCode::Digit1),
            1 => Some(KeyCode::Digit2),
            2 => Some(KeyCode::Digit3),
            3 => Some(KeyCode::Digit4),
            4 => Some(KeyCode::Digit5),
            5 => Some(KeyCode::Digit6),
            6 => Some(KeyCode::Digit7),
            7 => Some(KeyCode::Digit8),
            8 => Some(KeyCode::Digit9),
            _ => None,
        }
    }

    fn numpad_key_for_option(option_index: usize) -> Option<KeyCode> {
        match option_index {
            0 => Some(KeyCode::Numpad1),
            1 => Some(KeyCode::Numpad2),
            2 => Some(KeyCode::Numpad3),
            3 => Some(KeyCode::Numpad4),
            4 => Some(KeyCode::Numpad5),
            5 => Some(KeyCode::Numpad6),
            6 => Some(KeyCode::Numpad7),
            7 => Some(KeyCode::Numpad8),
            8 => Some(KeyCode::Numpad9),
            _ => None,
        }
    }

    fn lever_click_pong_actions(option_count: usize) -> Vec<Vec<LeverActions>> {
        (0..option_count)
            .filter_map(|option_index| {
                Self::lever_action_for_option(option_index).map(|action| vec![action])
            })
            .collect()
    }

    fn estimate_wrapped_lines(text: &str) -> usize {
        text.lines()
            .map(|line| {
                let chars = line.chars().count().max(1);
                chars.div_ceil(Self::OPTION_WINDOW_CHARS_PER_LINE_ESTIMATE)
            })
            .sum::<usize>()
            .max(1)
    }

    fn estimate_option_window_height(title: &str, description: &str) -> f32 {
        let title_lines = Self::estimate_wrapped_lines(title);
        let description_lines = Self::estimate_wrapped_lines(description);
        let line_count = title_lines + description_lines;
        let text_height = line_count as f32 * Self::OPTION_WINDOW_LINE_HEIGHT_ESTIMATE;
        let padding_height = Self::OPTION_WINDOW_VERTICAL_PADDING * 2.0;
        (Self::OPTION_WINDOW_HEADER_HEIGHT + text_height + padding_height)
            .max(Self::OPTION_WINDOW_MIN_HEIGHT_ESTIMATE)
    }

    fn option_window_translation(
        option_index: usize,
        option_count: usize,
        _option_window_height: f32,
    ) -> Vec3 {
        let row = option_index / Self::OPTION_WINDOW_COLUMNS;
        let col = option_index % Self::OPTION_WINDOW_COLUMNS;

        let x = if col == 0 {
            Self::OPTION_WINDOW_LEFT_X
        } else {
            Self::OPTION_WINDOW_RIGHT_X
        };
        let mut y = Self::OPTION_WINDOW_TOP_Y + row as f32 * Self::OPTION_WINDOW_ROW_SPACING;
        if option_index % 2 == 1 {
            let extra_options = option_count.saturating_sub(2) as f32;
            y -= 75.0 * extra_options;
        }

        Vec3::new(x, y, Self::OPTION_WINDOW_Z)
    }

    fn setup(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        stage: Res<DilemmaStage>,
        index: Res<CurrentDilemmaStageIndex>,
        mut lever: ResMut<Lever>,
    ) {
        let selected_option = if index.0 == 0 {
            stage.default_option
        } else {
            lever.selected_index().or(stage.default_option)
        };
        let state = LeverState::from_option_index(selected_option);
        let option_count = stage.options.len().max(1);
        lever.set_state_and_options(state, option_count);
        let color = selected_option.map_or(Color::WHITE, option_color);
        let initial_click_state = selected_option.unwrap_or(0).min(option_count.saturating_sub(1));
        let click_actions = Self::lever_click_pong_actions(option_count);

        let mut option_key_mappings = Vec::new();
        for option_index in 0..stage.options.len() {
            let Some(action) = Self::lever_action_for_option(option_index) else {
                continue;
            };
            let Some(digit_key) = Self::digit_key_for_option(option_index) else {
                continue;
            };
            let mut keys = vec![digit_key];
            if let Some(numpad_key) = Self::numpad_key_for_option(option_index) {
                keys.push(numpad_key);
            }
            option_key_mappings.push(KeyMapping {
                keys,
                actions: vec![action],
                allow_repeated_activation: false,
            });
        }

        commands
            .spawn((
                DespawnOnExit(DilemmaPhase::Decision),
                DecisionScene,
                children![
                    ContinuousAudioPallet::new(vec![
                        ContinuousAudio {
                            key: DilemmaSounds::TrainApproaching,
                            source: AudioPlayer::<AudioSource>(
                                asset_server.load("./audio/effects/train/approaching.ogg")
                            ),
                            settings: PlaybackSettings {
                                volume: Volume::Linear(1.0),
                                ..continuous_audio()
                            },
                            dilatable: true
                        },
                        ContinuousAudio {
                            key: DilemmaSounds::Clock,
                            source: AudioPlayer::<AudioSource>(
                                asset_server.load("./audio/effects/clock.ogg")
                            ),
                            settings: PlaybackSettings {
                                volume: Volume::Linear(0.3),
                                ..continuous_audio()
                            },
                            dilatable: true
                        }
                    ]),
                    (
                        Pressable::new(vec![KeyMapping {
                            keys: vec![KeyCode::Enter],
                            actions: vec![DecisionActions::LockDecision],
                            allow_repeated_activation: false
                        }]),
                        ActionPallet(enum_map!(
                            DecisionActions::LockDecision => vec![
                                InputAction::ChangeState(
                                    StateVector::new(
                                        None, None, Some(DilemmaPhase::Skip)
                                    ),
                                ),
                                InputAction::PlaySound(DilemmaSounds::Lever)
                            ]
                        ))
                    ),
                    (
                        TextTitle,
                        DilemmaTimer::new(
                            stage.countdown_duration,
                            Duration::from_secs_f32(5.0),
                            Duration::from_secs_f32(2.0)
                        ),
                        ColorAnchor::default(),
                        ColorChangeOn::new(vec![ColorChangeEvent::Pulse(vec![DANGER_COLOR])]),
                        Transform::from_translation(Self::TIMER_TRANSLATION)
                    ),
                    (
                        LeverRoot,
                        ClickablePong::new(click_actions, initial_click_state)
                            .with_region(Self::LEVER_CLICK_REGION),
                        Pressable::new(option_key_mappings),
                        ActionPallet(enum_map!(
                            LeverActions::SelectOption1 => vec![
                                InputAction::SetLeverSelection(Some(0)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption2 => vec![
                                InputAction::SetLeverSelection(Some(1)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption3 => vec![
                                InputAction::SetLeverSelection(Some(2)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption4 => vec![
                                InputAction::SetLeverSelection(Some(3)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption5 => vec![
                                InputAction::SetLeverSelection(Some(4)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption6 => vec![
                                InputAction::SetLeverSelection(Some(5)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption7 => vec![
                                InputAction::SetLeverSelection(Some(6)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption8 => vec![
                                InputAction::SetLeverSelection(Some(7)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                            LeverActions::SelectOption9 => vec![
                                InputAction::SetLeverSelection(Some(8)),
                                InputAction::PlaySound(DilemmaSounds::Lever),
                            ],
                        )),
                        CenterLever,
                        Text2d::new(LEVER_BASE),
                        TextFont {
                            font_size: scaled_font_size(12.0),
                            ..default()
                        },
                        TextColor(color),
                        TextLayout {
                            justify: Justify::Center,
                            ..default()
                        },
                        TransientAudioPallet::new(vec![(
                            DilemmaSounds::Lever,
                            vec![TransientAudio::new(
                                asset_server.load("./audio/effects/switch.ogg"),
                                0.1,
                                true,
                                1.0,
                                true
                            )]
                        )]),
                    )
                ],
            ))
            .with_children(|parent| {
                let option_count = stage.options.len();
                for (option_index, option) in stage.options.clone().into_iter().enumerate() {
                    let key_hint = Self::digit_key_for_option(option_index).map_or(
                        String::new(),
                        |_| format!(" [Press {} to select]", option_index + 1),
                    );
                    let title_text =
                        format!("Option {}: {}{}\n", option_index + 1, option.name, key_hint);
                    let window_height =
                        Self::estimate_option_window_height(&title_text, &option.description);
                    parent.spawn((
                        TextWindow {
                            title: Some(UiWindowTitle {
                                text: title_text,
                                ..default()
                            }),
                            ..default()
                        },
                        TextBounds {
                            width: Some(Self::OPTION_WINDOW_WIDTH),
                            height: None,
                        },
                        Draggable::default(),
                        TextColor(PRIMARY_COLOR),
                        Text2d::new(&option.description),
                        TextFont {
                            font_size: scaled_font_size(12.0),
                            ..default()
                        },
                        Anchor::TOP_LEFT,
                        Transform::from_translation(Self::option_window_translation(
                            option_index,
                            option_count,
                            window_height,
                        )),
                    ));
                }
            });
    }

    fn cleanup(
        mut commands: Commands,
        background_query: Query<Entity, With<Background>>,
        track_query: Query<Entity, With<Track>>,
    ) {
        for entity in background_query.iter() {
            commands.entity(entity).insert(ColorTranslation::new(
                Color::NONE,
                Duration::from_secs_f32(0.4),
                false,
            ));
        }
        for entity in track_query.iter() {
            commands.entity(entity).insert(ColorTranslation::new(
                Color::NONE,
                Duration::from_secs_f32(0.4),
                false,
            ));
        }
    }

    fn update_stats(
        mut stats: ResMut<DilemmaStats>,
        lever: Res<Lever>,
        mut timer: Query<&mut DilemmaTimer>,
    ) {
        for timer in timer.iter_mut() {
            stats.update(&lever.0, &timer.timer);
        }
    }

    fn finalize_stats(
        mut stats: ResMut<DilemmaStats>,
        lever: Res<Lever>,
        stage: Res<DilemmaStage>,
        mut timer: Query<&mut DilemmaTimer>,
    ) {
        let Some(selected_option) = lever
            .selected_index()
            .or(stage.default_option)
            .and_then(|option_index| stage.options.get(option_index))
        else {
            return;
        };
        let consequence = selected_option.consequences;

        for timer in timer.iter_mut() {
            stats.finalize(&consequence, &lever.0, &timer.timer);
        }
    }

    fn sync_pong_state_to_lever(
        lever: Res<Lever>,
        mut lever_query: Query<
            (
                &mut ClickablePong<LeverActions>,
                &mut Clickable<LeverActions>,
                &mut InteractionState,
            ),
            With<LeverRoot>,
        >,
    ) {
        let Some(selected_index) = lever.selected_index() else {
            return;
        };

        for (mut pong, mut clickable, mut interaction_state) in lever_query.iter_mut() {
            pong.synchronize_index(&mut clickable, &mut interaction_state, selected_index);
        }
    }

    fn update_screen_shake(
        mut screen_shake: ResMut<ScreenShakeState>,
        train_translation_query: Query<&PointToPointTranslation, With<Train>>,
    ) {
        let Some(translation) = train_translation_query.iter().next() else {
            screen_shake.target_intensity = 0.0;
            return;
        };

        let progress = translation.timer.fraction().clamp(0.0, 1.0);
        let proximity_curve = progress.powf(2.4);
        screen_shake.target_intensity = (proximity_curve * 3.12).clamp(0.0, 1.0);
    }

    fn clear_screen_shake(mut screen_shake: ResMut<ScreenShakeState>) {
        screen_shake.target_intensity = 0.0;
    }
}
