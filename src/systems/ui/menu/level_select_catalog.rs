use std::collections::HashSet;

use crate::scenes::dilemma::content::{
    DilemmaPathDeontological, DilemmaPathInaction, DilemmaPathUtilitarian, DilemmaScene,
    Lab0Dilemma, Lab1Dilemma, Lab2Dilemma, Lab3Dilemma, Lab4Dilemma,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct LevelSelectNodeId(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LevelSelectFileNode {
    pub id: LevelSelectNodeId,
    pub file_name: &'static str,
    pub scene: DilemmaScene,
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

fn file(id: &'static str, file_name: &'static str, scene: DilemmaScene) -> LevelSelectNode {
    LevelSelectNode::File(LevelSelectFileNode {
        id: LevelSelectNodeId(id),
        file_name,
        scene,
    })
}

pub(super) fn level_select_catalog_root() -> LevelSelectFolderNode {
    LevelSelectFolderNode {
        id: LevelSelectNodeId("root"),
        label: "LEVEL SELECT",
        children: vec![
            file(
                "lab_0_incompetent_bandit",
                "lab0_incompetent_bandit.dilem",
                DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit),
            ),
            file(
                "lab_1_near_sighted_bandit",
                "lab1_near_sighted_bandit.dilem",
                DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit),
            ),
            file(
                "lab_2_the_trolley_problem",
                "lab2_the_trolley_problem.dilem",
                DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem),
            ),
            folder(
                "path_inaction",
                "path_inaction",
                vec![
                    file(
                        "path_inaction_0",
                        "path_inaction_empty_choice.dilem",
                        DilemmaScene::PathInaction(DilemmaPathInaction::EmptyChoice, 0),
                    ),
                    file(
                        "path_inaction_1",
                        "path_inaction_plenty_of_time.dilem",
                        DilemmaScene::PathInaction(DilemmaPathInaction::PlentyOfTime, 1),
                    ),
                    file(
                        "path_inaction_2",
                        "path_inaction_little_time.dilem",
                        DilemmaScene::PathInaction(DilemmaPathInaction::LittleTime, 2),
                    ),
                    file(
                        "path_inaction_3",
                        "path_inaction_five_or_nothing.dilem",
                        DilemmaScene::PathInaction(DilemmaPathInaction::FiveOrNothing, 3),
                    ),
                    file(
                        "path_inaction_4",
                        "path_inaction_a_cure_for_cancer.dilem",
                        DilemmaScene::PathInaction(DilemmaPathInaction::CancerCure, 4),
                    ),
                    file(
                        "path_inaction_5",
                        "path_inaction_your_own_child.dilem",
                        DilemmaScene::PathInaction(DilemmaPathInaction::OwnChild, 5),
                    ),
                    file(
                        "path_inaction_6",
                        "path_inaction_you.dilem",
                        DilemmaScene::PathInaction(DilemmaPathInaction::You, 6),
                    ),
                ],
            ),
            file(
                "lab_3_asleep_at_the_job",
                "lab3_asleep_at_the_job.dilem",
                DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob),
            ),
            folder(
                "path_deontological",
                "path_deontological",
                vec![
                    file(
                        "path_deontological_0",
                        "path_deontological_trolleyer_problem.dilem",
                        DilemmaScene::PathDeontological(
                            DilemmaPathDeontological::TrolleyerProblem,
                            0,
                        ),
                    ),
                    file(
                        "path_deontological_1",
                        "path_deontological_trolleyest_problem.dilem",
                        DilemmaScene::PathDeontological(
                            DilemmaPathDeontological::TrolleyestProblem,
                            1,
                        ),
                    ),
                    file(
                        "path_deontological_2",
                        "path_deontological_trolleygeddon_problem.dilem",
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
                    file(
                        "path_utilitarian_0",
                        "path_utilitarian_one_fifth.dilem",
                        DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::OneFifth, 0),
                    ),
                    file(
                        "path_utilitarian_1",
                        "path_utilitarian_margin_of_error.dilem",
                        DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::MarginOfError, 1),
                    ),
                    file(
                        "path_utilitarian_2",
                        "path_utilitarian_negligible_difference.dilem",
                        DilemmaScene::PathUtilitarian(
                            DilemmaPathUtilitarian::NegligibleDifference,
                            2,
                        ),
                    ),
                    file(
                        "path_utilitarian_3",
                        "path_utilitarian_unorthodox_surgery.dilem",
                        DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::UnorthodoxSurgery, 3),
                    ),
                ],
            ),
            file(
                "lab_4_random_deaths",
                "lab4_random_deaths.dilem",
                DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths),
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
                if query_match(file.file_name, query) {
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
    use super::*;

    #[test]
    fn flattened_default_rows_include_all_current_dilemmas() {
        let rows = default_level_select_file_rows();
        assert_eq!(rows.len(), 19);
        assert!(matches!(
            rows.first(),
            Some(LevelSelectVisibleRow {
                label: "lab0_incompetent_bandit.dilem",
                ..
            })
        ));
        assert!(matches!(
            rows.last(),
            Some(LevelSelectVisibleRow {
                label: "lab4_random_deaths.dilem",
                ..
            })
        ));
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
        assert!(rows.iter().any(|row| row.label == "path_utilitarian_unorthodox_surgery.dilem"
            && matches!(row.kind, LevelSelectVisibleRowKind::File(_))));
    }

    #[test]
    fn query_projection_is_case_insensitive() {
        let root = level_select_catalog_root();
        let expansion = LevelSelectExpansionState::default();
        let rows = visible_rows_for_query(&root, &expansion, "LAB4_RANDOM_DEATHS");

        assert!(rows.iter().any(|row| row.label == "lab4_random_deaths.dilem"));
    }
}
