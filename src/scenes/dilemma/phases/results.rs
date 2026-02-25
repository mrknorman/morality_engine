use std::{collections::HashMap, time::Duration};

use bevy::{audio::Volume, prelude::*};
use enum_map::{enum_map, Enum};

use crate::{
    data::{
        states::DilemmaPhase,
        stats::{DilemmaRunStatsScope, DilemmaStats, GameStats},
    },
    entities::{
        large_fonts::{AsciiString, TextEmotion},
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, Table, TextButton, WindowedTable},
        train::Train,
    },
    scenes::dilemma::DilemmaSounds,
    style::common_ui::NextButton,
    systems::{
        audio::{continuous_audio, MusicAudio, TransientAudio, TransientAudioPallet},
        backgrounds::{content::BackgroundTypes, Background},
        colors::{ColorTranslation, DIM_BACKGROUND_COLOR, PRIMARY_COLOR},
        inheritance::BequeathTextColor,
        interaction::{ActionPallet, Draggable, InputAction, SelectableClickActivation},
        particles::FireworkLauncher,
        physics::Velocity,
        ui::{
            tabs::{TabBar, TabBarState},
            window::{
                UiWindow, UiWindowContent, UiWindowContentMetrics, UiWindowOverflowPolicy,
                UiWindowTabRow, UiWindowTitle,
            },
        },
    },
};

pub struct DilemmaResultsPlugin;
impl Plugin for DilemmaResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(DilemmaPhase::Results), DilemmaResultsScene::setup)
            .add_systems(
                Update,
                DilemmaResultsScene::sync_multi_stage_tab_visibility
                    .run_if(in_state(DilemmaPhase::Results)),
            );
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaResultsActions {
    ExitResults,
}

impl std::fmt::Display for DilemmaResultsActions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DilemmaResultsScene;

#[derive(Component, Clone, Copy)]
struct MultiStageResultsTabTable {
    window_entity: Entity,
    tab_index: usize,
}

impl DilemmaResultsScene {
    const TEXT_BOX_Z: f32 = 1.0;

    fn table_size(table: &Table) -> Vec2 {
        let width = table
            .columns
            .iter()
            .fold(0.0, |acc, column| acc + column.width.max(1.0));
        let height = table
            .rows
            .iter()
            .fold(0.0, |acc, row| acc + row.height.max(1.0));
        Vec2::new(width, height)
    }

    fn spawn_latest_results_window(commands: &mut Commands, parent: Entity, latest: DilemmaStats) {
        commands.entity(parent).with_children(|scene| {
            scene.spawn((
                Draggable::default(),
                WindowedTable {
                    title: Some(UiWindowTitle {
                        text: String::from("Latest Results"),
                        ..default()
                    }),
                    ..default()
                },
                latest.to_table(),
                Transform::from_xyz(-450.0, 0.0, Self::TEXT_BOX_Z + 0.2),
            ));
        });
    }

    fn spawn_multi_stage_results_window(
        commands: &mut Commands,
        parent: Entity,
        run_entries: &[DilemmaStats],
    ) {
        let mut tab_tables: Vec<(Table, Vec2)> = Vec::with_capacity(run_entries.len() + 1);
        let summary = GameStats::from_dilemma_stats(run_entries).to_table();
        tab_tables.push((summary, Vec2::ZERO));
        for entry in run_entries {
            tab_tables.push((entry.to_table(), Vec2::ZERO));
        }
        for (table, size) in tab_tables.iter_mut() {
            *size = Self::table_size(table);
        }

        let max_table_size = tab_tables.iter().fold(Vec2::ZERO, |acc, (_, size)| {
            Vec2::new(acc.x.max(size.x), acc.y.max(size.y))
        });
        let tab_labels: Vec<String> = std::iter::once(String::from("All"))
            .chain((1..=run_entries.len()).map(|index| index.to_string()))
            .collect();
        let tab_count = tab_labels.len().max(1) as f32;
        let tab_width = (560.0 / tab_count).clamp(22.0, 44.0);
        let tab_row_height = 28.0;
        let tab_total_width = tab_width * tab_count;

        let window_size = Vec2::new(
            (max_table_size.x + 26.0).max(tab_total_width),
            max_table_size.y + tab_row_height + 30.0,
        );
        let table_top_y = window_size.y * 0.5 - tab_row_height - 8.0;

        let window_entity = commands
            .spawn((
                Name::new("multi_stage_results_window"),
                Draggable::default(),
                UiWindow::new(
                    Some(UiWindowTitle {
                        text: String::from("Decision Results"),
                        ..default()
                    }),
                    HollowRectangle {
                        dimensions: window_size,
                        thickness: 2.0,
                        color: PRIMARY_COLOR,
                        ..default()
                    },
                    20.0,
                    true,
                    None,
                ),
                UiWindowContentMetrics::from_min_inner(window_size),
                UiWindowOverflowPolicy::ConstrainToContent,
                UiWindowTabRow {
                    labels: tab_labels,
                    tab_width,
                    row_height: tab_row_height,
                    text_size: scaled_font_size(11.0),
                    selected_text_size: scaled_font_size(16.0),
                    color: PRIMARY_COLOR,
                    z: 0.24,
                    click_activation: SelectableClickActivation::HoveredOnly,
                },
                Transform::from_xyz(-450.0, 0.0, Self::TEXT_BOX_Z + 0.2),
            ))
            .id();

        let content_root = commands
            .spawn((
                Name::new("multi_stage_results_window_content"),
                UiWindowContent::new(window_entity),
                Transform::default(),
            ))
            .id();
        commands.entity(window_entity).add_child(content_root);

        commands.entity(content_root).with_children(|content| {
            for (tab_index, (table, table_size)) in tab_tables.into_iter().enumerate() {
                content.spawn((
                    Name::new(format!("multi_stage_results_tab_table_{tab_index}")),
                    MultiStageResultsTabTable {
                        window_entity,
                        tab_index,
                    },
                    table,
                    if tab_index == 0 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    Transform::from_xyz(-table_size.x * 0.5, table_top_y, 0.2),
                ));
            }
        });

        commands.entity(parent).add_child(window_entity);
    }

    fn setup(
        mut commands: Commands,
        mut train_query: Query<(&mut Transform, &mut Velocity), With<Train>>,
        stats: Res<GameStats>,
        run_scope: Res<DilemmaRunStatsScope>,
        asset_server: Res<AssetServer>,
    ) {
        let run_entries: Vec<DilemmaStats> = {
            let scoped = run_scope.entries(&stats);
            if scoped.is_empty() {
                stats.dilemma_stats.last().cloned().into_iter().collect()
            } else {
                scoped.to_vec()
            }
        };
        let is_multi_stage = run_scope.expected_stage_count > 1 && run_entries.len() > 1;

        let scene_entity = commands
            .spawn((
                Self,
                DespawnOnExit(DilemmaPhase::Results),
                children![
                    (
                        FireworkLauncher::new(100.0, 0.2, 5.0),
                        Transform::from_xyz(-500., -500., -10.)
                    ),
                    (
                        FireworkLauncher::new(100.0, 0.2, 5.0),
                        Transform::from_xyz(500., -500., -10.)
                    ),
                    (
                        Draggable::default(),
                        WindowedTable {
                            title: Some(UiWindowTitle {
                                text: String::from("Results"),
                                ..default()
                            }),
                            ..default()
                        },
                        stats.to_table(),
                        Transform::from_xyz(50.0, 0.0, Self::TEXT_BOX_Z + 0.2,)
                    ),
                    (
                        TextColor(Color::NONE),
                        Background::new(BackgroundTypes::Desert, 0.00002, -0.5),
                        BequeathTextColor,
                        ColorTranslation::new(
                            DIM_BACKGROUND_COLOR,
                            Duration::from_secs_f32(0.2),
                            false
                        )
                    ),
                    (
                        MusicAudio,
                        AudioPlayer::<AudioSource>(
                            asset_server.load("./audio/music/the_right_track.ogg")
                        ),
                        PlaybackSettings {
                            paused: false,
                            volume: Volume::Linear(0.3),
                            ..continuous_audio()
                        }
                    ),
                    (
                        TextColor(PRIMARY_COLOR),
                        TextEmotion::Happy,
                        AsciiString("DILEMMA RESULTS".to_string()),
                        Transform::from_xyz(0.0, 300.0, 1.0)
                    ),
                    (
                        NextButton,
                        TextButton::new(
                            vec![DilemmaResultsActions::ExitResults],
                            vec![KeyCode::Enter],
                            "[ Click here or Press Enter to End the Simulation ]",
                        ),
                        ActionPallet::<DilemmaResultsActions, DilemmaSounds>(enum_map!(
                            DilemmaResultsActions::ExitResults => vec![
                                InputAction::PlaySound(DilemmaSounds::Click),
                                InputAction::NextScene,
                                InputAction::Despawn(None)
                        ])),
                        TransientAudioPallet::new(vec![(
                            DilemmaSounds::Click,
                            vec![TransientAudio::new(
                                asset_server.load("./audio/effects/mech_click.ogg"),
                                0.1,
                                true,
                                1.0,
                                true
                            )]
                        )])
                    )
                ],
            ))
            .id();

        if is_multi_stage {
            Self::spawn_multi_stage_results_window(&mut commands, scene_entity, &run_entries);
        } else {
            let latest = run_entries
                .last()
                .cloned()
                .or_else(|| stats.dilemma_stats.last().cloned());
            if let Some(latest) = latest {
                Self::spawn_latest_results_window(&mut commands, scene_entity, latest);
            } else {
                warn!("no dilemma stats available to populate latest results window");
            }
        }

        for (mut transform, mut velocity) in train_query.iter_mut() {
            velocity.0 = Vec3::ZERO;
            transform.translation = Vec3::new(120.0, 150.0, 0.0);
        }
    }

    fn sync_multi_stage_tab_visibility(
        tab_state_query: Query<(&TabBarState, &ChildOf), With<TabBar>>,
        mut table_query: Query<(&MultiStageResultsTabTable, &mut Visibility)>,
    ) {
        let mut active_index_by_window: HashMap<Entity, usize> = HashMap::new();
        for (tab_state, parent) in tab_state_query.iter() {
            active_index_by_window.insert(parent.parent(), tab_state.active_index);
        }

        for (tab_table, mut visibility) in table_query.iter_mut() {
            let active_index = active_index_by_window
                .get(&tab_table.window_entity)
                .copied()
                .unwrap_or(0);
            *visibility = if tab_table.tab_index == active_index {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
