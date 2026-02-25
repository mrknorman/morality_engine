use crate::{
    data::stats::{DilemmaStats, GameStats},
    scenes::Scene,
};

pub mod engine;
pub mod ids;
pub mod schema;
pub mod validate;
pub mod visualize;

pub fn next_scenes_for_current_dilemma(
    current_scene: Scene,
    latest: &DilemmaStats,
    stats: &GameStats,
) -> Option<Vec<Scene>> {
    match engine::evaluate_next_scenes_from_graph(current_scene, latest, stats) {
        Ok(next_scenes) => next_scenes,
        Err(error) => {
            bevy::log::warn!(
                "graph-driven flow evaluation failed; falling back to menu route: {}",
                error
            );
            Some(vec![Scene::Menu])
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::scenes::{
        dialogue::content::{
            DialogueScene, Lab1aDialogue, Lab1bDialogue, Lab2aDialogue, Lab2bDialogue,
            Lab3aDialogue, Lab3bDialogue, Lab4Dialogue, PathOutcome, PsychopathDialogue,
        },
        dilemma::content::{
            DilemmaScene, Lab0Dilemma, Lab1Dilemma, Lab2Dilemma, Lab3Dilemma, Lab4Dilemma,
        },
        dilemma::lever::LeverState,
        ending::content::EndingScene,
    };

    #[test]
    fn lab_three_junction_without_five_fatalities_routes_to_utilitarian_path() {
        let latest = DilemmaStats {
            num_fatalities: 1,
            ..Default::default()
        };
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::PassUtilitarian)),
                Scene::Dialogue(DialogueScene::Lab3b(Lab3bDialogue::Intro)),
                Scene::Dilemma(scene)
            ] if *scene == DilemmaScene::PATH_UTILITARIAN[0]
        ));
    }

    #[test]
    fn utilitarian_path_with_no_selection_defaults_to_fail_route() {
        let latest = DilemmaStats {
            result: Some(LeverState::Selected(0)),
            ..Default::default()
        };
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[0]),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::UtilitarianPath(_))),
                Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
                Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths))
            ]
        ));
    }

    #[test]
    fn random_deaths_scene_has_no_followup_route() {
        let latest = DilemmaStats::default();
        let stats = GameStats::default();

        assert!(next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
            &latest,
            &stats,
        )
        .is_none());
    }

    #[test]
    fn graph_routes_lab0_to_pass_path_when_no_decision_or_fatality() {
        let latest = DilemmaStats {
            result: Some(LeverState::Selected(0)),
            ..Default::default()
        };
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit)),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Pass)),
                Scene::Dialogue(_),
                Scene::Dilemma(_)
            ]
        ));
    }

    fn baseline_route_for_unchanged_branches(
        current_scene: Scene,
        latest: &DilemmaStats,
        stats: &GameStats,
    ) -> Option<Vec<Scene>> {
        match current_scene {
            Scene::Dilemma(DilemmaScene::Lab0(_)) => Some(baseline_lab_one(latest, stats)),
            Scene::Dilemma(DilemmaScene::Lab1(_)) => Some(baseline_lab_two(latest, stats)),
            Scene::Dilemma(DilemmaScene::PathInaction(_, stage)) => {
                Some(baseline_inaction_path(latest, stats, stage + 1))
            }
            Scene::Dilemma(DilemmaScene::Lab2(_)) => Some(baseline_lab_three(latest, stats)),
            Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)) => {
                Some(baseline_lab_three_junction(latest, stats))
            }
            Scene::Dilemma(DilemmaScene::PathUtilitarian(_, stage)) => {
                Some(baseline_utilitarian_path(latest, stats, stage + 1))
            }
            Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)) => None,
            _ => None,
        }
    }

    fn baseline_lab_one(latest: &DilemmaStats, _: &GameStats) -> Vec<Scene> {
        if latest.num_fatalities > 0 {
            vec![
                Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Fail)),
                Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[0]),
            ]
        } else if latest.num_decisions > 0 {
            if let Some(duration) = latest.duration_remaining_at_last_decision {
                if latest.num_decisions > 10 {
                    vec![
                        Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::FailVeryIndecisive)),
                        Scene::Ending(EndingScene::Leverophile),
                    ]
                } else if duration < Duration::from_secs(1) {
                    vec![
                        Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::PassSlow)),
                        Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
                        Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit)),
                    ]
                } else {
                    vec![
                        Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::PassIndecisive)),
                        Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
                        Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit)),
                    ]
                }
            } else {
                vec![
                    Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::PassIndecisive)),
                    Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
                    Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit)),
                ]
            }
        } else {
            vec![
                Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Pass)),
                Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
                Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit)),
            ]
        }
    }

    fn baseline_lab_two(latest: &DilemmaStats, stats: &GameStats) -> Vec<Scene> {
        if latest.num_fatalities > 0 {
            if latest.num_decisions > 0 {
                vec![
                    Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::FailIndecisive)),
                    Scene::Ending(EndingScene::Leverophile),
                ]
            } else if stats.total_decisions == 0 {
                vec![
                    Scene::Dialogue(DialogueScene::path_inaction(0, PathOutcome::Fail)),
                    Scene::Dilemma(DilemmaScene::PATH_INACTION[0]),
                ]
            } else {
                vec![
                    Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::Fail)),
                    Scene::Ending(EndingScene::ImpatientPsychopath),
                ]
            }
        } else if latest.num_decisions > 0 {
            if let (Some(duration), Some(average_duration)) = (
                latest.duration_remaining_at_last_decision,
                stats.overall_avg_time_remaining,
            ) {
                if average_duration < Duration::from_secs(1) {
                    vec![
                        Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::PassSlowAgain)),
                        Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
                        Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
                    ]
                } else if duration < Duration::from_secs(1) {
                    vec![
                        Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::PassSlow)),
                        Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
                        Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
                    ]
                } else {
                    vec![
                        Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::Pass)),
                        Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
                        Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
                    ]
                }
            } else {
                vec![
                    Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::Pass)),
                    Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
                    Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
                ]
            }
        } else {
            vec![
                Scene::Dialogue(DialogueScene::Lab2a(Lab2aDialogue::Pass)),
                Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
                Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
            ]
        }
    }

    fn baseline_lab_three(latest: &DilemmaStats, _: &GameStats) -> Vec<Scene> {
        if latest.num_fatalities == 5 {
            if latest.num_decisions > 0 {
                vec![Scene::Dialogue(DialogueScene::Lab3a(
                    Lab3aDialogue::FailIndecisive,
                ))]
            } else {
                vec![
                    Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::FailInaction)),
                    Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)),
                ]
            }
        } else {
            vec![
                Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::PassUtilitarian)),
                Scene::Dialogue(DialogueScene::Lab3b(Lab3bDialogue::Intro)),
                Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[0]),
            ]
        }
    }

    fn baseline_lab_three_junction(latest: &DilemmaStats, _: &GameStats) -> Vec<Scene> {
        if latest.num_fatalities == 5 {
            if latest.num_decisions > 0 {
                vec![Scene::Dialogue(DialogueScene::Lab3a(
                    Lab3aDialogue::FailIndecisive,
                ))]
            } else {
                vec![
                    Scene::Dialogue(DialogueScene::path_deontological(0, PathOutcome::Fail)),
                    Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[0]),
                ]
            }
        } else {
            vec![
                Scene::Dialogue(DialogueScene::Lab3a(Lab3aDialogue::PassUtilitarian)),
                Scene::Dialogue(DialogueScene::Lab3b(Lab3bDialogue::Intro)),
                Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[0]),
            ]
        }
    }

    fn baseline_inaction_path(_: &DilemmaStats, stats: &GameStats, stage: usize) -> Vec<Scene> {
        if stats.total_decisions > 0 && stage < 6 {
            vec![
                Scene::Dialogue(DialogueScene::path_inaction(stage, PathOutcome::Pass)),
                Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
                Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
            ]
        } else if stage < DilemmaScene::PATH_INACTION.len() {
            vec![
                Scene::Dialogue(DialogueScene::path_inaction(stage, PathOutcome::Fail)),
                Scene::Dilemma(DilemmaScene::PATH_INACTION[stage]),
            ]
        } else {
            vec![Scene::Ending(EndingScene::TrueNeutral)]
        }
    }

    fn baseline_utilitarian_path(latest: &DilemmaStats, _: &GameStats, stage: usize) -> Vec<Scene> {
        let selected_option = latest.result.and_then(|state| state.to_int()).unwrap_or(0);

        match (selected_option, stage) {
            (0, 4) => vec![
                Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Pass)),
                Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
                Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
            ],

            (_, 4) => vec![
                Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Fail)),
                Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
                Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
            ],

            (selected, next_stage) if selected > 0 => vec![
                Scene::Dialogue(DialogueScene::path_utilitarian(
                    next_stage,
                    PathOutcome::Pass,
                )),
                Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[next_stage]),
            ],

            (0, _) => vec![
                Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Fail)),
                Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
                Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
            ],
            _ => unreachable!("utilitarian path should resolve from selected option index"),
        }
    }

    fn assert_graph_matches_baseline(scene: Scene, latest: DilemmaStats, stats: GameStats) {
        let baseline = baseline_route_for_unchanged_branches(scene, &latest, &stats);
        let graph = engine::evaluate_next_scenes_from_graph(scene, &latest, &stats)
            .expect("graph evaluation should succeed");
        let scene_label = match scene {
            Scene::Dilemma(DilemmaScene::Lab0(_)) => "lab0",
            Scene::Dilemma(DilemmaScene::Lab1(_)) => "lab1",
            Scene::Dilemma(DilemmaScene::Lab2(_)) => "lab2",
            Scene::Dilemma(DilemmaScene::Lab3(_)) => "lab3",
            Scene::Dilemma(DilemmaScene::PathInaction(_, stage)) => match stage {
                0 => "path_inaction.0",
                1 => "path_inaction.1",
                2 => "path_inaction.2",
                3 => "path_inaction.3",
                4 => "path_inaction.4",
                5 => "path_inaction.5",
                6 => "path_inaction.6",
                _ => "path_inaction.unknown",
            },
            Scene::Dilemma(DilemmaScene::PathUtilitarian(_, stage)) => match stage {
                0 => "path_utilitarian.0",
                1 => "path_utilitarian.1",
                2 => "path_utilitarian.2",
                3 => "path_utilitarian.3",
                _ => "path_utilitarian.unknown",
            },
            Scene::Dilemma(DilemmaScene::Lab4(_)) => "lab4",
            _ => "unsupported",
        };

        assert!(
            graph == baseline,
            "graph/baseline mismatch for {scene_label}; fatalities={}, decisions={}, total_decisions={}, result={:?}, last_remaining={:?}, overall_avg={:?}",
            latest.num_fatalities,
            latest.num_decisions,
            stats.total_decisions,
            latest.result,
            latest.duration_remaining_at_last_decision,
            stats.overall_avg_time_remaining
        );
    }

    #[test]
    fn graph_parity_matches_pre_json_baseline_for_unchanged_branches() {
        let scenes = {
            let mut entries = vec![
                Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit)),
                Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit)),
                Scene::Dilemma(DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem)),
                Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)),
                Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
            ];
            for scene in DilemmaScene::PATH_INACTION {
                entries.push(Scene::Dilemma(scene));
            }
            for scene in DilemmaScene::PATH_UTILITARIAN {
                entries.push(Scene::Dilemma(scene));
            }
            entries
        };

        let fatalities_values = [0usize, 1, 5];
        let decisions_values = [0usize, 1, 2, 11];
        let total_decisions_values = [0usize, 1, 3];
        let result_values = [
            None,
            Some(LeverState::Selected(0)),
            Some(LeverState::Selected(1)),
        ];
        let last_remaining_values = [
            None,
            Some(Duration::from_secs_f32(0.5)),
            Some(Duration::from_secs_f32(2.0)),
        ];
        let overall_avg_values = [
            None,
            Some(Duration::from_secs_f32(0.5)),
            Some(Duration::from_secs_f32(2.0)),
        ];

        for scene in scenes {
            for num_fatalities in fatalities_values {
                for num_decisions in decisions_values {
                    for total_decisions in total_decisions_values {
                        for result in result_values {
                            for last_remaining in last_remaining_values {
                                for overall_avg in overall_avg_values {
                                    let latest = DilemmaStats {
                                        num_fatalities,
                                        num_decisions,
                                        result,
                                        duration_remaining_at_last_decision: last_remaining,
                                        ..Default::default()
                                    };
                                    let stats = GameStats {
                                        total_decisions,
                                        overall_avg_time_remaining: overall_avg,
                                        ..Default::default()
                                    };

                                    assert_graph_matches_baseline(scene, latest, stats);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn deontological_acting_at_any_stage_routes_to_pass_and_selective_ending() {
        let stats = GameStats::default();

        for stage in [0usize, 1, 2] {
            let latest = DilemmaStats {
                num_decisions: 1,
                result: Some(LeverState::Selected(1)),
                ..Default::default()
            };

            let next = next_scenes_for_current_dilemma(
                Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[stage]),
                &latest,
                &stats,
            )
            .expect("expected a route");

            assert!(matches!(
                next.as_slice(),
                [
                    Scene::Dialogue(dialogue),
                    Scene::Ending(EndingScene::SelectiveDeontologist)
                ] if *dialogue == DialogueScene::path_deontological(1, PathOutcome::Pass)
            ));
        }
    }

    #[test]
    fn deontological_indecisive_action_routes_to_fail_indecisive_dialogue() {
        let stats = GameStats::default();

        for stage in [0usize, 1, 2] {
            let latest = DilemmaStats {
                num_decisions: 2,
                result: Some(LeverState::Selected(0)),
                ..Default::default()
            };

            let next = next_scenes_for_current_dilemma(
                Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[stage]),
                &latest,
                &stats,
            )
            .expect("expected a route");

            assert!(matches!(
                next.as_slice(),
                [
                    Scene::Dialogue(DialogueScene::Lab3a(
                        Lab3aDialogue::DeontologicalFailIndecisive
                    )),
                    Scene::Ending(EndingScene::ConfusedDeontologist)
                ]
            ));
        }
    }

    #[test]
    fn deontological_final_stage_without_action_routes_to_true_deontologist() {
        let latest = DilemmaStats::default();
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[2]),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(dialogue),
                Scene::Ending(EndingScene::TrueDeontologist)
            ] if *dialogue == DialogueScene::path_deontological(3, PathOutcome::Fail)
        ));
    }

    #[test]
    fn deontological_early_stages_without_action_continue_fail_chain() {
        let latest = DilemmaStats::default();
        let stats = GameStats::default();

        let stage_zero_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[0]),
            &latest,
            &stats,
        )
        .expect("expected a route");
        assert!(matches!(
            stage_zero_next.as_slice(),
            [
                Scene::Dialogue(dialogue),
                Scene::Dilemma(next_scene)
            ] if *dialogue == DialogueScene::path_deontological(1, PathOutcome::Fail)
                && *next_scene == DilemmaScene::PATH_DEONTOLOGICAL[1]
        ));

        let stage_one_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[1]),
            &latest,
            &stats,
        )
        .expect("expected a route");
        assert!(matches!(
            stage_one_next.as_slice(),
            [
                Scene::Dialogue(dialogue),
                Scene::Dilemma(next_scene)
            ] if *dialogue == DialogueScene::path_deontological(2, PathOutcome::Fail)
                && *next_scene == DilemmaScene::PATH_DEONTOLOGICAL[2]
        ));
    }

    #[test]
    fn day_personal_special_route_ordering_matches_intended_priority() {
        let scene = Scene::Dilemma(DilemmaScene::DAY_PERSONAL[2]);

        let did_nothing =
            next_scenes_for_current_dilemma(scene, &DilemmaStats::default(), &GameStats::default())
                .expect("expected a route");
        assert!(matches!(
            did_nothing.as_slice(),
            [Scene::Ending(EndingScene::DayPersonalDidNothing)]
        ));

        let ignored_bomb = next_scenes_for_current_dilemma(
            scene,
            &DilemmaStats::default(),
            &GameStats {
                total_decisions: 1,
                ..Default::default()
            },
        )
        .expect("expected a route");
        assert!(matches!(
            ignored_bomb.as_slice(),
            [Scene::Ending(EndingScene::DayPersonalIgnoredBomb)]
        ));

        let all_men = next_scenes_for_current_dilemma(
            scene,
            &DilemmaStats {
                num_decisions: 1,
                result: Some(LeverState::Selected(0)),
                ..Default::default()
            },
            &GameStats {
                total_decisions: 1,
                ..Default::default()
            },
        )
        .expect("expected a route");
        assert!(matches!(
            all_men.as_slice(),
            [Scene::Ending(EndingScene::DayPersonalAllMenKilled)]
        ));

        let all_women = next_scenes_for_current_dilemma(
            scene,
            &DilemmaStats {
                num_decisions: 1,
                result: Some(LeverState::Selected(1)),
                ..Default::default()
            },
            &GameStats {
                total_decisions: 1,
                ..Default::default()
            },
        )
        .expect("expected a route");
        assert!(matches!(
            all_women.as_slice(),
            [Scene::Ending(EndingScene::DayPersonalAllWomenKilled)]
        ));
    }

    #[test]
    fn psychopath_branch_starts_after_lab0_fatality() {
        let latest = DilemmaStats {
            num_fatalities: 1,
            ..Default::default()
        };
        let stats = GameStats::default();

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit)),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Fail)),
                Scene::Dilemma(next_scene)
            ] if *next_scene == DilemmaScene::PATH_PSYCHOPATH[0]
        ));
    }

    #[test]
    fn psychopath_try_again_routes_fail_or_rejoin_main_loop() {
        let stats = GameStats::default();

        let fail_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[0]),
            &DilemmaStats {
                num_fatalities: 1,
                ..Default::default()
            },
            &stats,
        )
        .expect("expected a route");
        assert!(matches!(
            fail_next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::PathPsychopath(
                    PsychopathDialogue::TryAgainFail
                )),
                Scene::Dilemma(next_scene)
            ] if *next_scene == DilemmaScene::PATH_PSYCHOPATH[1]
        ));

        let pass_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[0]),
            &DilemmaStats::default(),
            &stats,
        )
        .expect("expected a route");
        assert!(matches!(
            pass_next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::PathPsychopath(
                    PsychopathDialogue::TryAgainPass
                )),
                Scene::Dialogue(DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro)),
                Scene::Dilemma(DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit))
            ]
        ));
    }

    #[test]
    fn psychopath_baby_one_requires_previous_one_and_baby() {
        let latest = DilemmaStats {
            result: Some(LeverState::Selected(1)),
            ..Default::default()
        };
        let stats_one_then_baby = GameStats {
            dilemma_stats: vec![DilemmaStats {
                result: Some(LeverState::Selected(0)),
                ..Default::default()
            }],
            ..Default::default()
        };

        let baby_one_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[2]),
            &latest,
            &stats_one_then_baby,
        )
        .expect("expected a route");
        assert!(matches!(
            baby_one_next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::PathPsychopath(PsychopathDialogue::BabyOne)),
                Scene::Dilemma(next_scene)
            ] if *next_scene == DilemmaScene::PATH_PSYCHOPATH[3]
        ));

        let stats_two_then_baby = GameStats {
            dilemma_stats: vec![DilemmaStats {
                result: Some(LeverState::Selected(1)),
                ..Default::default()
            }],
            ..Default::default()
        };
        let baby_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[2]),
            &latest,
            &stats_two_then_baby,
        )
        .expect("expected a route");
        assert!(matches!(
            baby_next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::PathPsychopath(PsychopathDialogue::Baby)),
                Scene::Dilemma(next_scene)
            ] if *next_scene == DilemmaScene::PATH_PSYCHOPATH[3]
        ));
    }

    #[test]
    fn psychopath_city_max_death_requires_two_then_nuns_then_city() {
        let latest = DilemmaStats {
            result: Some(LeverState::Selected(0)),
            ..Default::default()
        };
        let stats = GameStats {
            dilemma_stats: vec![
                DilemmaStats {
                    result: Some(LeverState::Selected(1)),
                    ..Default::default()
                },
                DilemmaStats {
                    result: Some(LeverState::Selected(0)),
                    ..Default::default()
                },
                DilemmaStats {
                    num_decisions: 0,
                    result: Some(LeverState::Selected(0)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[4]),
            &latest,
            &stats,
        )
        .expect("expected a route");

        assert!(matches!(
            next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::PathPsychopath(
                    PsychopathDialogue::CityMaxDeath
                )),
                Scene::Ending(EndingScene::IdioticPsychopath)
            ]
        ));
    }

    #[test]
    fn psychopath_pain_routes_distinguish_slow_from_repentant_chain() {
        let latest = DilemmaStats {
            result: Some(LeverState::Selected(1)),
            ..Default::default()
        };

        let slow_stats = GameStats {
            dilemma_stats: vec![
                DilemmaStats {
                    result: Some(LeverState::Selected(0)),
                    ..Default::default()
                },
                DilemmaStats {
                    result: Some(LeverState::Selected(1)),
                    ..Default::default()
                },
                DilemmaStats {
                    num_decisions: 1,
                    result: Some(LeverState::Selected(0)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let slow_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[4]),
            &latest,
            &slow_stats,
        )
        .expect("expected a route");
        assert!(matches!(
            slow_next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::PathPsychopath(
                    PsychopathDialogue::PainMaxPain
                )),
                Scene::Ending(EndingScene::IdioticPsychopath)
            ]
        ));

        let repentant_stats = GameStats {
            dilemma_stats: vec![
                DilemmaStats {
                    result: Some(LeverState::Selected(0)),
                    ..Default::default()
                },
                DilemmaStats {
                    result: Some(LeverState::Selected(1)),
                    ..Default::default()
                },
                DilemmaStats {
                    num_decisions: 0,
                    result: Some(LeverState::Selected(0)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let repentant_next = next_scenes_for_current_dilemma(
            Scene::Dilemma(DilemmaScene::PATH_PSYCHOPATH[4]),
            &latest,
            &repentant_stats,
        )
        .expect("expected a route");
        assert!(matches!(
            repentant_next.as_slice(),
            [
                Scene::Dialogue(DialogueScene::PathPsychopath(
                    PsychopathDialogue::PainRepentant
                )),
                Scene::Ending(EndingScene::IdioticPsychopath)
            ]
        ));
    }
}
