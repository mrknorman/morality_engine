use std::collections::HashSet;

use crate::scenes::{
    dialogue::content::{
        DialogueScene, Lab0Dialogue, Lab1aDialogue, Lab1bDialogue, Lab2aDialogue, Lab2bDialogue,
        Lab3aDialogue, Lab3bDialogue, Lab4Dialogue, PathOutcome, PsychopathDialogue,
    },
    dilemma::content::{
        DilemmaDayPersonal, DilemmaPathDeontological, DilemmaPathInaction, DilemmaPathPsychopath,
        DilemmaPathUtilitarian, DilemmaScene, Lab0Dilemma, Lab1Dilemma, Lab2Dilemma, Lab3Dilemma,
        Lab4Dilemma,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct LevelSelectNodeId(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LevelSelectFileNode {
    pub id: LevelSelectNodeId,
    pub file_name: &'static str,
    pub scene: LevelSelectPlayableScene,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LevelSelectPlayableScene {
    Dilemma(DilemmaScene),
    Dialogue(DialogueScene),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LevelSelectFolderNode {
    pub id: LevelSelectNodeId,
    pub label: &'static str,
    pub children: Vec<LevelSelectNode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum LevelSelectNode {
    Folder(LevelSelectFolderNode),
    File(LevelSelectFileNode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum LevelSelectVisibleRowKind {
    Folder,
    File(LevelSelectFileNode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LevelSelectVisibleRow {
    pub id: LevelSelectNodeId,
    pub label: &'static str,
    pub depth: usize,
    pub kind: LevelSelectVisibleRowKind,
}

#[derive(Clone, Debug, Default)]
pub(super) struct LevelSelectExpansionState {
    expanded: HashSet<LevelSelectNodeId>,
}

impl LevelSelectExpansionState {
    #[cfg(test)]
    pub(super) fn all_expanded(root: &LevelSelectFolderNode) -> Self {
        let mut expanded = HashSet::new();
        collect_folder_ids(root, &mut expanded);
        Self { expanded }
    }

    pub(super) fn is_expanded(&self, id: LevelSelectNodeId) -> bool {
        self.expanded.contains(&id)
    }

    pub(super) fn set_expanded(&mut self, id: LevelSelectNodeId, expanded: bool) {
        if expanded {
            self.expanded.insert(id);
        } else {
            self.expanded.remove(&id);
        }
    }

    pub(super) fn toggle(&mut self, id: LevelSelectNodeId) {
        let expand = !self.is_expanded(id);
        self.set_expanded(id, expand);
    }
}

fn folder(
    id: &'static str,
    label: &'static str,
    children: impl Into<Vec<LevelSelectNode>>,
) -> LevelSelectNode {
    LevelSelectNode::Folder(LevelSelectFolderNode {
        id: LevelSelectNodeId(id),
        label,
        children: children.into(),
    })
}

fn dilemma_file(id: &'static str, file_name: &'static str, scene: DilemmaScene) -> LevelSelectNode {
    LevelSelectNode::File(LevelSelectFileNode {
        id: LevelSelectNodeId(id),
        file_name,
        scene: LevelSelectPlayableScene::Dilemma(scene),
    })
}

fn dialogue_file(
    id: &'static str,
    file_name: &'static str,
    scene: DialogueScene,
) -> LevelSelectNode {
    LevelSelectNode::File(LevelSelectFileNode {
        id: LevelSelectNodeId(id),
        file_name,
        scene: LevelSelectPlayableScene::Dialogue(scene),
    })
}

pub(super) fn level_select_catalog_root() -> LevelSelectFolderNode {
    // Keep this catalog aligned with graph-driven campaign flow.
    // The test `graph_referenced_scenes_are_in_level_select_catalog` enforces that every
    // dialogue/dilemma referenced in `src/scenes/flow/content/campaign_graph.json` has
    // a corresponding level-select entry.
    LevelSelectFolderNode {
        id: LevelSelectNodeId("root"),
        label: "LEVEL SELECT",
        children: vec![
            folder(
                "dilemmas",
                "dilemmas",
                vec![
                    dilemma_file(
                        "lab_0_incompetent_bandit",
                        "lab0_incompetent_bandit.dilem",
                        DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit),
                    ),
                    dilemma_file(
                        "lab_1_near_sighted_bandit",
                        "lab1_near_sighted_bandit.dilem",
                        DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit),
                    ),
                    dilemma_file(
                        "lab_2_the_trolley_problem",
                        "lab2_the_trolley_problem.dilem",
                        DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem),
                    ),
                    folder(
                        "path_inaction",
                        "path_inaction",
                        vec![
                            dilemma_file(
                                "path_inaction_0",
                                "empty_choice.dilem",
                                DilemmaScene::PathInaction(DilemmaPathInaction::EmptyChoice, 0),
                            ),
                            dilemma_file(
                                "path_inaction_1",
                                "plenty_of_time.dilem",
                                DilemmaScene::PathInaction(DilemmaPathInaction::PlentyOfTime, 1),
                            ),
                            dilemma_file(
                                "path_inaction_2",
                                "little_time.dilem",
                                DilemmaScene::PathInaction(DilemmaPathInaction::LittleTime, 2),
                            ),
                            dilemma_file(
                                "path_inaction_3",
                                "five_or_nothing.dilem",
                                DilemmaScene::PathInaction(DilemmaPathInaction::FiveOrNothing, 3),
                            ),
                            dilemma_file(
                                "path_inaction_4",
                                "a_cure_for_cancer.dilem",
                                DilemmaScene::PathInaction(DilemmaPathInaction::CancerCure, 4),
                            ),
                            dilemma_file(
                                "path_inaction_5",
                                "your_own_child.dilem",
                                DilemmaScene::PathInaction(DilemmaPathInaction::OwnChild, 5),
                            ),
                            dilemma_file(
                                "path_inaction_6",
                                "you.dilem",
                                DilemmaScene::PathInaction(DilemmaPathInaction::You, 6),
                            ),
                        ],
                    ),
                    folder(
                        "path_psychopath",
                        "path_psychopath",
                        vec![
                            dilemma_file(
                                "path_psychopath_0",
                                "try_again.dilem",
                                DilemmaScene::PathPsychopath(DilemmaPathPsychopath::TryAgain, 0),
                            ),
                            dilemma_file(
                                "path_psychopath_1",
                                "one_or_two.dilem",
                                DilemmaScene::PathPsychopath(DilemmaPathPsychopath::OneOrTwo, 1),
                            ),
                            dilemma_file(
                                "path_psychopath_2",
                                "death_at_a_convent.dilem",
                                DilemmaScene::PathPsychopath(
                                    DilemmaPathPsychopath::DeathAtAConvent,
                                    2,
                                ),
                            ),
                            dilemma_file(
                                "path_psychopath_3",
                                "prolonged_suffering.dilem",
                                DilemmaScene::PathPsychopath(
                                    DilemmaPathPsychopath::ProlongedSuffering,
                                    3,
                                ),
                            ),
                            dilemma_file(
                                "path_psychopath_4",
                                "train_of_mass_destruction.dilem",
                                DilemmaScene::PathPsychopath(
                                    DilemmaPathPsychopath::TrainOfMassDestruction,
                                    4,
                                ),
                            ),
                        ],
                    ),
                    dilemma_file(
                        "lab_3_asleep_at_the_job",
                        "lab3_asleep_at_the_job.dilem",
                        DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob),
                    ),
                    folder(
                        "path_deontological",
                        "path_deontological",
                        vec![
                            dilemma_file(
                                "path_deontological_0",
                                "trolleyer_problem.dilem",
                                DilemmaScene::PathDeontological(
                                    DilemmaPathDeontological::TrolleyerProblem,
                                    0,
                                ),
                            ),
                            dilemma_file(
                                "path_deontological_1",
                                "trolleyest_problem.dilem",
                                DilemmaScene::PathDeontological(
                                    DilemmaPathDeontological::TrolleyestProblem,
                                    1,
                                ),
                            ),
                            dilemma_file(
                                "path_deontological_2",
                                "trolleygeddon_problem.dilem",
                                DilemmaScene::PathDeontological(
                                    DilemmaPathDeontological::TrolleygeddonProblem,
                                    2,
                                ),
                            ),
                        ],
                    ),
                    folder(
                        "path_utilitarian",
                        "path_utilitarian",
                        vec![
                            dilemma_file(
                                "path_utilitarian_0",
                                "one_fifth.dilem",
                                DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::OneFifth, 0),
                            ),
                            dilemma_file(
                                "path_utilitarian_1",
                                "margin_of_error.dilem",
                                DilemmaScene::PathUtilitarian(
                                    DilemmaPathUtilitarian::MarginOfError,
                                    1,
                                ),
                            ),
                            dilemma_file(
                                "path_utilitarian_2",
                                "negligible_difference.dilem",
                                DilemmaScene::PathUtilitarian(
                                    DilemmaPathUtilitarian::NegligibleDifference,
                                    2,
                                ),
                            ),
                            dilemma_file(
                                "path_utilitarian_3",
                                "unorthodox_surgery.dilem",
                                DilemmaScene::PathUtilitarian(
                                    DilemmaPathUtilitarian::UnorthodoxSurgery,
                                    3,
                                ),
                            ),
                        ],
                    ),
                    dilemma_file(
                        "lab_4_random_deaths",
                        "lab4_random_deaths.dilem",
                        DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths),
                    ),
                    folder(
                        "day_personal",
                        "day_personal",
                        vec![
                            dilemma_file(
                                "day_personal_0_george_or_hannah",
                                "0_george_or_hannah.dilem",
                                DilemmaScene::DayPersonal(DilemmaDayPersonal::GeorgeOrHannah, 0),
                            ),
                            dilemma_file(
                                "day_personal_1_twist_of_fate",
                                "1_twist_of_fate.dilem",
                                DilemmaScene::DayPersonal(DilemmaDayPersonal::TwistOfFate, 1),
                            ),
                            dilemma_file(
                                "day_personal_2_decision_ignition",
                                "2_decision_ignition.dilem",
                                DilemmaScene::DayPersonal(DilemmaDayPersonal::DecisionIgnition, 2),
                            ),
                            dilemma_file(
                                "day_personal_3_authorized_worker_vs_partying_youths",
                                "3_authorized_worker_vs_partying_youths.dilem",
                                DilemmaScene::DayPersonal(
                                    DilemmaDayPersonal::AuthorizedWorkerVsPartyingYouths,
                                    3,
                                ),
                            ),
                            dilemma_file(
                                "day_personal_4_fat_man_on_bridge",
                                "4_fat_man_on_bridge.dilem",
                                DilemmaScene::DayPersonal(DilemmaDayPersonal::FatManOnBridge, 4),
                            ),
                            dilemma_file(
                                "day_personal_5_fat_construction_worker_vs_partying_youths",
                                "5_fat_construction_worker_vs_partying_youths.dilem",
                                DilemmaScene::DayPersonal(
                                    DilemmaDayPersonal::FatConstructionWorkerVsPartyingYouths,
                                    5,
                                ),
                            ),
                            dilemma_file(
                                "day_personal_6_boulder_vs_death_cult",
                                "6_boulder_vs_death_cult.dilem",
                                DilemmaScene::DayPersonal(
                                    DilemmaDayPersonal::BoulderVsDeathCult,
                                    6,
                                ),
                            ),
                            dilemma_file(
                                "day_personal_7_cable_car_medalist_vs_skiers",
                                "7_cable_car_medalist_vs_skiers.dilem",
                                DilemmaScene::DayPersonal(
                                    DilemmaDayPersonal::CableCarMedalistVsSkiers,
                                    7,
                                ),
                            ),
                        ],
                    ),
                ],
            ),
            folder(
                "chat_logs",
                "chat_logs",
                vec![
                    folder(
                        "dialogue_lab_0",
                        "lab_0",
                        vec![dialogue_file(
                            "dialogue_lab_0_intro",
                            "intro",
                            DialogueScene::Lab0(Lab0Dialogue::Intro),
                        )],
                    ),
                    folder(
                        "dialogue_lab_1",
                        "lab_1",
                        vec![
                            folder(
                                "dialogue_lab_1_a",
                                "a",
                                vec![
                                    dialogue_file(
                                        "dialogue_lab_1_a_fail",
                                        "fail",
                                        DialogueScene::Lab1a(Lab1aDialogue::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_1_a_pass_indecisive",
                                        "pass_indecisive",
                                        DialogueScene::Lab1a(Lab1aDialogue::PassIndecisive),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_1_a_fail_very_indecisive",
                                        "fail_very_indecisive",
                                        DialogueScene::Lab1a(Lab1aDialogue::FailVeryIndecisive),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_1_a_pass",
                                        "pass",
                                        DialogueScene::Lab1a(Lab1aDialogue::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_1_a_pass_slow",
                                        "pass_slow",
                                        DialogueScene::Lab1a(Lab1aDialogue::PassSlow),
                                    ),
                                ],
                            ),
                            folder(
                                "dialogue_lab_1_b",
                                "b",
                                vec![dialogue_file(
                                    "dialogue_lab_1_b_intro",
                                    "intro",
                                    DialogueScene::Lab1b(Lab1bDialogue::DilemmaIntro),
                                )],
                            ),
                            folder(
                                "dialogue_path_psychopath",
                                "path_psychopath",
                                vec![
                                    folder(
                                        "dialogue_path_psychopath_0",
                                        "0",
                                        vec![
                                            dialogue_file(
                                                "dialogue_path_psychopath_0_fail",
                                                "fail",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::TryAgainFail,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_0_pass",
                                                "pass",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::TryAgainPass,
                                                ),
                                            ),
                                        ],
                                    ),
                                    folder(
                                        "dialogue_path_psychopath_1",
                                        "1",
                                        vec![
                                            dialogue_file(
                                                "dialogue_path_psychopath_1_one",
                                                "one",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::One,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_1_two",
                                                "two",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::Two,
                                                ),
                                            ),
                                        ],
                                    ),
                                    folder(
                                        "dialogue_path_psychopath_2",
                                        "2",
                                        vec![
                                            dialogue_file(
                                                "dialogue_path_psychopath_2_baby_one",
                                                "baby_one",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::BabyOne,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_2_baby",
                                                "baby",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::Baby,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_2_nuns",
                                                "nuns",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::Nuns,
                                                ),
                                            ),
                                        ],
                                    ),
                                    folder(
                                        "dialogue_path_psychopath_3",
                                        "3",
                                        vec![
                                            dialogue_file(
                                                "dialogue_path_psychopath_3_fast_repentant",
                                                "fast_repentant",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::FastRepentant,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_3_fast",
                                                "fast",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::Fast,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_3_slow",
                                                "slow",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::Slow,
                                                ),
                                            ),
                                        ],
                                    ),
                                    folder(
                                        "dialogue_path_psychopath_4",
                                        "4",
                                        vec![
                                            dialogue_file(
                                                "dialogue_path_psychopath_4_city_max_death",
                                                "city_max_death",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::CityMaxDeath,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_4_city_repentant",
                                                "city_repentant",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::CityRepentant,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_4_city",
                                                "city",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::City,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_4_pain_max_pain",
                                                "pain_max_pain",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::PainMaxPain,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_4_pain_repentant",
                                                "pain_repentant",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::PainRepentant,
                                                ),
                                            ),
                                            dialogue_file(
                                                "dialogue_path_psychopath_4_pain",
                                                "pain",
                                                DialogueScene::PathPsychopath(
                                                    PsychopathDialogue::Pain,
                                                ),
                                            ),
                                        ],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    folder(
                        "dialogue_lab_2",
                        "lab_2",
                        vec![
                            folder(
                                "dialogue_lab_2_a",
                                "a",
                                vec![
                                    dialogue_file(
                                        "dialogue_lab_2_a_fail_indecisive",
                                        "fail_indecisive",
                                        DialogueScene::Lab2a(Lab2aDialogue::FailIndecisive),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_2_a_fail",
                                        "fail",
                                        DialogueScene::Lab2a(Lab2aDialogue::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_2_a_pass_slow_again",
                                        "pass_slow_again",
                                        DialogueScene::Lab2a(Lab2aDialogue::PassSlowAgain),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_2_a_pass_slow",
                                        "pass_slow",
                                        DialogueScene::Lab2a(Lab2aDialogue::PassSlow),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_2_a_pass",
                                        "pass",
                                        DialogueScene::Lab2a(Lab2aDialogue::Pass),
                                    ),
                                ],
                            ),
                            folder(
                                "dialogue_lab_2_b",
                                "b",
                                vec![dialogue_file(
                                    "dialogue_lab_2_b_intro",
                                    "intro",
                                    DialogueScene::Lab2b(Lab2bDialogue::Intro),
                                )],
                            ),
                            folder(
                                "dialogue_path_inaction",
                                "path_inaction",
                                vec![
                                    dialogue_file(
                                        "dialogue_path_inaction_pass_1",
                                        "pass_1",
                                        DialogueScene::path_inaction(1, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_pass_2",
                                        "pass_2",
                                        DialogueScene::path_inaction(2, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_pass_3",
                                        "pass_3",
                                        DialogueScene::path_inaction(3, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_pass_4",
                                        "pass_4",
                                        DialogueScene::path_inaction(4, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_pass_5",
                                        "pass_5",
                                        DialogueScene::path_inaction(5, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_pass_6",
                                        "pass_6",
                                        DialogueScene::path_inaction(6, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_fail_0",
                                        "fail_0",
                                        DialogueScene::path_inaction(0, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_fail_1",
                                        "fail_1",
                                        DialogueScene::path_inaction(1, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_fail_2",
                                        "fail_2",
                                        DialogueScene::path_inaction(2, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_fail_3",
                                        "fail_3",
                                        DialogueScene::path_inaction(3, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_fail_4",
                                        "fail_4",
                                        DialogueScene::path_inaction(4, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_fail_5",
                                        "fail_5",
                                        DialogueScene::path_inaction(5, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_inaction_fail_6",
                                        "fail_6",
                                        DialogueScene::path_inaction(6, PathOutcome::Fail),
                                    ),
                                ],
                            ),
                        ],
                    ),
                    folder(
                        "dialogue_lab_3",
                        "lab_3",
                        vec![
                            folder(
                                "dialogue_lab_3_a",
                                "a",
                                vec![
                                    dialogue_file(
                                        "dialogue_lab_3_a_fail_indecisive",
                                        "fail_indecisive",
                                        DialogueScene::Lab3a(Lab3aDialogue::FailIndecisive),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_3_a_fail_inaction",
                                        "fail_inaction",
                                        DialogueScene::Lab3a(Lab3aDialogue::FailInaction),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_3_a_pass_utilitarian",
                                        "pass_utilitarian",
                                        DialogueScene::Lab3a(Lab3aDialogue::PassUtilitarian),
                                    ),
                                    dialogue_file(
                                        "dialogue_lab_3_a_deontological_fail_indecisive",
                                        "deontological_fail_indecisive",
                                        DialogueScene::Lab3a(
                                            Lab3aDialogue::DeontologicalFailIndecisive,
                                        ),
                                    ),
                                ],
                            ),
                            folder(
                                "dialogue_lab_3_b",
                                "b",
                                vec![dialogue_file(
                                    "dialogue_lab_3_b_intro",
                                    "intro",
                                    DialogueScene::Lab3b(Lab3bDialogue::Intro),
                                )],
                            ),
                            folder(
                                "dialogue_path_deontological",
                                "path_deontological",
                                vec![
                                    dialogue_file(
                                        "dialogue_path_deontological_pass",
                                        "pass",
                                        DialogueScene::path_deontological(1, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_deontological_fail_0",
                                        "fail_0",
                                        DialogueScene::path_deontological(0, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_deontological_fail_1",
                                        "fail_1",
                                        DialogueScene::path_deontological(1, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_deontological_fail_2",
                                        "fail_2",
                                        DialogueScene::path_deontological(2, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_deontological_fail_3",
                                        "fail_3",
                                        DialogueScene::path_deontological(3, PathOutcome::Fail),
                                    ),
                                ],
                            ),
                        ],
                    ),
                    folder(
                        "dialogue_lab_4",
                        "lab_4",
                        vec![
                            dialogue_file(
                                "dialogue_lab_4_outro",
                                "outro",
                                DialogueScene::Lab4(Lab4Dialogue::Outro),
                            ),
                            folder(
                                "dialogue_path_utilitarian",
                                "path_utilitarian",
                                vec![
                                    dialogue_file(
                                        "dialogue_path_utilitarian_1_pass",
                                        "1_pass",
                                        DialogueScene::path_utilitarian(1, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_utilitarian_1_fail",
                                        "1_fail",
                                        DialogueScene::path_utilitarian(1, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_utilitarian_2_pass",
                                        "2_pass",
                                        DialogueScene::path_utilitarian(2, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_utilitarian_2_fail",
                                        "2_fail",
                                        DialogueScene::path_utilitarian(2, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_utilitarian_3_pass",
                                        "3_pass",
                                        DialogueScene::path_utilitarian(3, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_utilitarian_3_fail",
                                        "3_fail",
                                        DialogueScene::path_utilitarian(3, PathOutcome::Fail),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_utilitarian_4_pass",
                                        "4_pass",
                                        DialogueScene::path_utilitarian(4, PathOutcome::Pass),
                                    ),
                                    dialogue_file(
                                        "dialogue_path_utilitarian_4_fail",
                                        "4_fail",
                                        DialogueScene::path_utilitarian(4, PathOutcome::Fail),
                                    ),
                                ],
                            ),
                        ],
                    ),
                ],
            ),
        ],
    }
}

pub(super) fn flatten_visible_rows(
    root: &LevelSelectFolderNode,
    expansion: &LevelSelectExpansionState,
) -> Vec<LevelSelectVisibleRow> {
    let mut rows = Vec::new();
    flatten_children(&root.children, 0, expansion, &mut rows);
    rows
}

pub(super) fn visible_rows_for_query(
    root: &LevelSelectFolderNode,
    expansion: &LevelSelectExpansionState,
    normalized_query: &str,
) -> Vec<LevelSelectVisibleRow> {
    let query = normalized_query.trim();
    if query.is_empty() {
        return flatten_visible_rows(root, expansion);
    }

    let mut rows = Vec::new();
    let query = query.to_ascii_lowercase();
    flatten_children_matching_query(&root.children, 0, &query, &mut rows);
    rows
}

#[cfg(test)]
pub(super) fn default_level_select_file_rows() -> Vec<LevelSelectVisibleRow> {
    let root = level_select_catalog_root();
    let expansion = LevelSelectExpansionState::all_expanded(&root);
    flatten_visible_rows(&root, &expansion)
        .into_iter()
        .filter(|row| matches!(row.kind, LevelSelectVisibleRowKind::File(_)))
        .collect()
}

fn flatten_children(
    children: &[LevelSelectNode],
    depth: usize,
    expansion: &LevelSelectExpansionState,
    rows: &mut Vec<LevelSelectVisibleRow>,
) {
    for node in children {
        match node {
            LevelSelectNode::Folder(folder) => {
                rows.push(LevelSelectVisibleRow {
                    id: folder.id,
                    label: folder.label,
                    depth,
                    kind: LevelSelectVisibleRowKind::Folder,
                });
                if expansion.is_expanded(folder.id) {
                    flatten_children(&folder.children, depth + 1, expansion, rows);
                }
            }
            LevelSelectNode::File(file) => {
                rows.push(LevelSelectVisibleRow {
                    id: file.id,
                    label: file.file_name,
                    depth,
                    kind: LevelSelectVisibleRowKind::File(*file),
                });
            }
        }
    }
}

fn flatten_children_matching_query(
    children: &[LevelSelectNode],
    depth: usize,
    query: &str,
    rows: &mut Vec<LevelSelectVisibleRow>,
) -> bool {
    let mut matched_any = false;

    for node in children {
        match node {
            LevelSelectNode::Folder(folder) => {
                let folder_matches = query_match(folder.label, query);
                let mut child_rows = Vec::new();
                let child_matches = flatten_children_matching_query(
                    &folder.children,
                    depth + 1,
                    query,
                    &mut child_rows,
                );
                if folder_matches || child_matches {
                    rows.push(LevelSelectVisibleRow {
                        id: folder.id,
                        label: folder.label,
                        depth,
                        kind: LevelSelectVisibleRowKind::Folder,
                    });
                    rows.extend(child_rows);
                    matched_any = true;
                }
            }
            LevelSelectNode::File(file) => {
                if file_matches_query(file, query) {
                    rows.push(LevelSelectVisibleRow {
                        id: file.id,
                        label: file.file_name,
                        depth,
                        kind: LevelSelectVisibleRowKind::File(*file),
                    });
                    matched_any = true;
                }
            }
        }
    }

    matched_any
}

fn query_match(label: &str, query: &str) -> bool {
    label.to_ascii_lowercase().contains(query)
}

fn file_matches_query(file: &LevelSelectFileNode, query: &str) -> bool {
    query_match(file.file_name, query)
        || matches!(file.scene, LevelSelectPlayableScene::Dialogue(_))
            && query_match(&format!("{}.log", file.file_name), query)
}

#[cfg(test)]
fn collect_folder_ids(folder: &LevelSelectFolderNode, expanded: &mut HashSet<LevelSelectNodeId>) {
    expanded.insert(folder.id);
    for child in &folder.children {
        if let LevelSelectNode::Folder(child_folder) = child {
            collect_folder_ids(child_folder, expanded);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scenes::{
        flow::{
            engine,
            schema::{SceneProgressionGraph, SceneRef},
        },
        Scene,
    };

    use super::*;

    const CAMPAIGN_GRAPH_JSON: &str =
        include_str!("../../../scenes/flow/content/campaign_graph.json");

    #[test]
    fn flattened_default_rows_include_dilemma_and_dialogue_entries() {
        let rows = default_level_select_file_rows();
        assert!(rows.len() > 30);
        assert!(rows
            .iter()
            .any(|row| row.label == "lab0_incompetent_bandit.dilem"));
        assert!(rows.iter().any(|row| row.label == "empty_choice.dilem"));
        assert!(rows.iter().any(|row| row.label == "intro"));
    }

    #[test]
    fn top_level_folders_are_dilemmas_and_chat_logs() {
        let root = level_select_catalog_root();
        let top_folders = root
            .children
            .iter()
            .filter_map(|node| match node {
                LevelSelectNode::Folder(folder) => Some(folder.label),
                LevelSelectNode::File(_) => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(top_folders, vec!["dilemmas", "chat_logs"]);
    }

    #[test]
    fn catalog_node_ids_are_unique() {
        let root = level_select_catalog_root();
        let expansion = LevelSelectExpansionState::all_expanded(&root);
        let rows = flatten_visible_rows(&root, &expansion);
        let mut ids = HashSet::new();
        for row in rows {
            assert!(ids.insert(row.id));
        }
    }

    #[test]
    fn query_projection_includes_folder_ancestors_for_matching_files() {
        let root = level_select_catalog_root();
        let expansion = LevelSelectExpansionState::default();
        let rows = visible_rows_for_query(&root, &expansion, "unorthodox");

        assert!(rows.iter().any(|row| row.label == "path_utilitarian"
            && matches!(row.kind, LevelSelectVisibleRowKind::Folder)));
        assert!(rows
            .iter()
            .any(|row| row.label == "unorthodox_surgery.dilem"
                && matches!(row.kind, LevelSelectVisibleRowKind::File(_))));
    }

    #[test]
    fn query_projection_is_case_insensitive() {
        let root = level_select_catalog_root();
        let expansion = LevelSelectExpansionState::default();
        let rows = visible_rows_for_query(&root, &expansion, "LAB4_RANDOM_DEATHS");

        assert!(rows
            .iter()
            .any(|row| row.label == "lab4_random_deaths.dilem"));
    }

    #[test]
    fn query_projection_matches_dialogue_log_prefix() {
        let root = level_select_catalog_root();
        let expansion = LevelSelectExpansionState::default();
        let rows = visible_rows_for_query(&root, &expansion, "pass_utilitarian.log");

        assert!(rows.iter().any(|row| row.label == "pass_utilitarian"
            && matches!(row.kind, LevelSelectVisibleRowKind::File(_))));
    }

    #[test]
    fn graph_referenced_scenes_are_in_level_select_catalog() {
        let graph: SceneProgressionGraph =
            serde_json::from_str(CAMPAIGN_GRAPH_JSON).expect("campaign graph should parse");

        let catalog_rows = default_level_select_file_rows();
        let catalog_scenes = catalog_rows
            .iter()
            .filter_map(|row| match row.kind {
                LevelSelectVisibleRowKind::File(file) => Some(file.scene),
                LevelSelectVisibleRowKind::Folder => None,
            })
            .collect::<Vec<_>>();

        let mut graph_refs = Vec::<SceneRef>::new();
        for route in &graph.routes {
            graph_refs.push(route.from.clone());
            for scene in &route.default_then {
                graph_refs.push(scene.clone());
            }
            for rule in &route.rules {
                for scene in &rule.then {
                    graph_refs.push(scene.clone());
                }
            }
        }

        for scene_ref in graph_refs {
            let expected = match &scene_ref {
                SceneRef::Dilemma { .. } | SceneRef::Dialogue { .. } => {
                    let runtime_scene = engine::runtime_scene_from_graph_ref(&scene_ref)
                        .expect("graph scene ref should map to runtime scene");
                    match runtime_scene {
                        Scene::Dilemma(scene) => LevelSelectPlayableScene::Dilemma(scene),
                        Scene::Dialogue(scene) => LevelSelectPlayableScene::Dialogue(scene),
                        Scene::Menu | Scene::Loading | Scene::Ending(_) => continue,
                    }
                }
                SceneRef::Menu | SceneRef::Loading | SceneRef::Ending { .. } => continue,
            };

            assert!(
                catalog_scenes.contains(&expected),
                "level select missing graph scene reference: {:?}",
                scene_ref
            );
        }
    }
}
