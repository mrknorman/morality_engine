use std::time::Duration;

use crate::{
    data::stats::{DilemmaStats, GameStats},
    scenes::{dialogue::content::*, dilemma::content::*, ending::content::*, Scene},
};

pub mod engine;
pub mod ids;
pub mod schema;
pub mod validate;

pub fn next_scenes_for_current_dilemma(
    current_scene: Scene,
    latest: &DilemmaStats,
    stats: &GameStats,
) -> Option<Vec<Scene>> {
    let baseline = next_scenes_hardcoded_for_current_dilemma(current_scene, latest, stats);
    match engine::evaluate_next_scenes_from_graph(current_scene, latest, stats) {
        Ok(Some(graph_scenes)) => {
            if baseline.as_ref() != Some(&graph_scenes) {
                bevy::log::warn!(
                    "scene-flow graph mismatch in shadow mode; scene_kind={}, fatalities={}, decisions={}, total_decisions={}, selected_option={:?}",
                    scene_kind_label(current_scene),
                    latest.num_fatalities,
                    latest.num_decisions,
                    stats.total_decisions,
                    latest.result.and_then(|state| state.to_int()),
                );
            }
        }
        Ok(None) => {}
        Err(error) => {
            bevy::log::warn!("graph-driven flow evaluation failed in shadow mode: {}", error);
        }
    }

    baseline
}

fn scene_kind_label(scene: Scene) -> &'static str {
    match scene {
        Scene::Menu => "menu",
        Scene::Loading => "loading",
        Scene::Dialogue(_) => "dialogue",
        Scene::Dilemma(_) => "dilemma",
        Scene::Ending(_) => "ending",
    }
}

fn next_scenes_hardcoded_for_current_dilemma(
    current_scene: Scene,
    latest: &DilemmaStats,
    stats: &GameStats,
) -> Option<Vec<Scene>> {
    match current_scene {
        Scene::Dilemma(DilemmaScene::Lab0(_)) => Some(lab_one(latest, stats)),
        Scene::Dilemma(DilemmaScene::Lab1(_)) => Some(lab_two(latest, stats)),
        Scene::Dilemma(DilemmaScene::PathInaction(_, stage)) => {
            Some(inaction_path(latest, stats, stage + 1))
        }
        Scene::Dilemma(DilemmaScene::Lab2(_)) => Some(lab_three(latest, stats)),
        Scene::Dilemma(DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob)) => {
            Some(lab_three_junction(latest, stats))
        }
        Scene::Dilemma(DilemmaScene::PathUtilitarian(_, stage)) => {
            Some(utilitarian_path(latest, stats, stage + 1))
        }
        Scene::Dilemma(DilemmaScene::PathDeontological(_, stage)) => {
            Some(deontological_path(latest, stats, stage + 1))
        }
        Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)) => None,
        _ => None,
    }
}

fn lab_one(latest: &DilemmaStats, _: &GameStats) -> Vec<Scene> {
    if latest.num_fatalities > 0 {
        vec![
            Scene::Dialogue(DialogueScene::Lab1a(Lab1aDialogue::Fail)),
            Scene::Ending(EndingScene::IdioticPsychopath),
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

fn lab_two(latest: &DilemmaStats, stats: &GameStats) -> Vec<Scene> {
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

fn lab_three(latest: &DilemmaStats, _: &GameStats) -> Vec<Scene> {
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

fn lab_three_junction(latest: &DilemmaStats, _: &GameStats) -> Vec<Scene> {
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

fn inaction_path(_: &DilemmaStats, stats: &GameStats, stage: usize) -> Vec<Scene> {
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

fn deontological_path(latest: &DilemmaStats, _: &GameStats, stage: usize) -> Vec<Scene> {
    if latest.num_fatalities == 1 && stage < 1 {
        vec![
            Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Pass)),
            Scene::Dialogue(DialogueScene::Lab2b(Lab2bDialogue::Intro)),
        ]
    } else if latest.num_fatalities == 1 && stage < 2 {
        vec![
            Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Pass)),
            Scene::Ending(EndingScene::SelectiveDeontologist),
        ]
    } else if stage < DilemmaScene::PATH_DEONTOLOGICAL.len() {
        vec![
            Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Fail)),
            Scene::Dilemma(DilemmaScene::PATH_DEONTOLOGICAL[stage]),
        ]
    } else {
        vec![
            Scene::Dialogue(DialogueScene::path_deontological(stage, PathOutcome::Fail)),
            Scene::Ending(EndingScene::TrueDeontologist),
        ]
    }
}

fn utilitarian_path(latest: &DilemmaStats, _: &GameStats, stage: usize) -> Vec<Scene> {
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

        (selected, stage) if selected > 0 => vec![
            Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Pass)),
            Scene::Dilemma(DilemmaScene::PATH_UTILITARIAN[stage]),
        ],

        (0, _) => vec![
            Scene::Dialogue(DialogueScene::path_utilitarian(stage, PathOutcome::Fail)),
            Scene::Dialogue(DialogueScene::Lab4(Lab4Dialogue::Outro)),
            Scene::Dilemma(DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths)),
        ],
        _ => unreachable!("utilitarian path should resolve from selected option index"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let latest = DilemmaStats::default();
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
    fn graph_shadow_parity_lab0_fail_route_matches_baseline() {
        let latest = DilemmaStats {
            num_fatalities: 1,
            ..Default::default()
        };
        let stats = GameStats::default();
        let scene = Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit));

        let baseline =
            next_scenes_hardcoded_for_current_dilemma(scene, &latest, &stats).expect("baseline");
        let graph = engine::evaluate_next_scenes_from_graph(scene, &latest, &stats)
            .expect("graph evaluation")
            .expect("graph route");

        assert!(graph == baseline);
    }

    #[test]
    fn graph_shadow_parity_lab0_decision_route_matches_baseline() {
        let latest = DilemmaStats {
            num_decisions: 1,
            duration_remaining_at_last_decision: Some(Duration::from_secs(2)),
            ..Default::default()
        };
        let stats = GameStats::default();
        let scene = Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit));

        let baseline =
            next_scenes_hardcoded_for_current_dilemma(scene, &latest, &stats).expect("baseline");
        let graph = engine::evaluate_next_scenes_from_graph(scene, &latest, &stats)
            .expect("graph evaluation")
            .expect("graph route");

        assert!(graph == baseline);
    }

    #[test]
    fn graph_shadow_parity_lab0_default_route_matches_baseline() {
        let latest = DilemmaStats::default();
        let stats = GameStats::default();
        let scene = Scene::Dilemma(DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit));

        let baseline =
            next_scenes_hardcoded_for_current_dilemma(scene, &latest, &stats).expect("baseline");
        let graph = engine::evaluate_next_scenes_from_graph(scene, &latest, &stats)
            .expect("graph evaluation")
            .expect("graph route");

        assert!(graph == baseline);
    }
}
