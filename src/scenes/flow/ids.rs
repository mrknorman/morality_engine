use super::schema::SceneRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathOutcomeId {
    Pass,
    Fail,
}

impl PathOutcomeId {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "pass" => Some(Self::Pass),
            "fail" => Some(Self::Fail),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueSceneId {
    Lab1aFail,
    Lab1aPassIndecisive,
    Lab1aFailVeryIndecisive,
    Lab1aPass,
    Lab1aPassSlow,
    Lab1bIntro,
    Lab2aFailIndecisive,
    Lab2aFail,
    Lab2aPassSlowAgain,
    Lab2aPassSlow,
    Lab2aPass,
    Lab2bIntro,
    Lab3aFailIndecisive,
    Lab3aFailInaction,
    Lab3aPassUtilitarian,
    Lab3bIntro,
    Lab4Outro,
    PathInaction { stage: u8, outcome: PathOutcomeId },
    PathDeontological { stage: u8, outcome: PathOutcomeId },
    PathUtilitarian { stage: u8, outcome: PathOutcomeId },
}

impl DialogueSceneId {
    pub fn parse(id: &str) -> Option<Self> {
        let static_id = match id {
            "lab_1.a.fail" => Some(Self::Lab1aFail),
            "lab_1.a.pass_indecisive" => Some(Self::Lab1aPassIndecisive),
            "lab_1.a.fail_very_indecisive" => Some(Self::Lab1aFailVeryIndecisive),
            "lab_1.a.pass" => Some(Self::Lab1aPass),
            "lab_1.a.pass_slow" => Some(Self::Lab1aPassSlow),
            "lab_1.b.intro" => Some(Self::Lab1bIntro),
            "lab_2.a.fail_indecisive" => Some(Self::Lab2aFailIndecisive),
            "lab_2.a.fail" => Some(Self::Lab2aFail),
            "lab_2.a.pass_slow_again" => Some(Self::Lab2aPassSlowAgain),
            "lab_2.a.pass_slow" => Some(Self::Lab2aPassSlow),
            "lab_2.a.pass" => Some(Self::Lab2aPass),
            "lab_2.b.intro" => Some(Self::Lab2bIntro),
            "lab_3.a.fail_indecisive" => Some(Self::Lab3aFailIndecisive),
            "lab_3.a.fail_inaction" => Some(Self::Lab3aFailInaction),
            "lab_3.a.pass_utilitarian" => Some(Self::Lab3aPassUtilitarian),
            "lab_3.b.intro" => Some(Self::Lab3bIntro),
            "lab_4.outro" => Some(Self::Lab4Outro),
            _ => None,
        };

        if static_id.is_some() {
            return static_id;
        }

        if let Some((stage, outcome)) = parse_path_with_outcome(id, "path_inaction", 0, 6) {
            return Some(Self::PathInaction { stage, outcome });
        }

        if let Some((stage, outcome)) = parse_path_with_outcome(id, "path_deontological", 0, 3) {
            return Some(Self::PathDeontological { stage, outcome });
        }

        if let Some((stage, outcome)) = parse_path_with_outcome(id, "path_utilitarian", 1, 4) {
            return Some(Self::PathUtilitarian { stage, outcome });
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DilemmaSceneId {
    Lab0IncompetentBandit,
    Lab1NearSightedBandit,
    Lab2TheTrolleyProblem,
    Lab3AsleepAtTheJob,
    Lab4RandomDeaths,
    PathInaction { stage: u8 },
    PathDeontological { stage: u8 },
    PathUtilitarian { stage: u8 },
}

impl DilemmaSceneId {
    pub fn parse(id: &str) -> Option<Self> {
        let static_id = match id {
            "lab_0.incompetent_bandit" => Some(Self::Lab0IncompetentBandit),
            "lab_1.near_sighted_bandit" => Some(Self::Lab1NearSightedBandit),
            "lab_2.the_trolley_problem" => Some(Self::Lab2TheTrolleyProblem),
            "lab_3.asleep_at_the_job" => Some(Self::Lab3AsleepAtTheJob),
            "lab_4.random_deaths" => Some(Self::Lab4RandomDeaths),
            _ => None,
        };

        if static_id.is_some() {
            return static_id;
        }

        if let Some(stage) = parse_path_index(id, "path_inaction", 0, 6) {
            return Some(Self::PathInaction { stage });
        }

        if let Some(stage) = parse_path_index(id, "path_deontological", 0, 2) {
            return Some(Self::PathDeontological { stage });
        }

        if let Some(stage) = parse_path_index(id, "path_utilitarian", 0, 3) {
            return Some(Self::PathUtilitarian { stage });
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndingSceneId {
    IdioticPsychopath,
    ImpatientPsychopath,
    Leverophile,
    SelectiveDeontologist,
    TrueDeontologist,
    TrueNeutral,
}

impl EndingSceneId {
    pub fn parse(id: &str) -> Option<Self> {
        match id {
            "idiotic_psychopath" => Some(Self::IdioticPsychopath),
            "impatient_psychopath" => Some(Self::ImpatientPsychopath),
            "leverophile" => Some(Self::Leverophile),
            "selective_deontologist" => Some(Self::SelectiveDeontologist),
            "true_deontologist" => Some(Self::TrueDeontologist),
            "true_neutral" => Some(Self::TrueNeutral),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypedSceneRef {
    Menu,
    Loading,
    Dialogue(DialogueSceneId),
    Dilemma(DilemmaSceneId),
    Ending(EndingSceneId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneIdParseError {
    UnknownDialogueId(String),
    UnknownDilemmaId(String),
    UnknownEndingId(String),
}

impl std::fmt::Display for SceneIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownDialogueId(id) => write!(f, "unknown dialogue id `{id}`"),
            Self::UnknownDilemmaId(id) => write!(f, "unknown dilemma id `{id}`"),
            Self::UnknownEndingId(id) => write!(f, "unknown ending id `{id}`"),
        }
    }
}

impl std::error::Error for SceneIdParseError {}

impl TryFrom<&SceneRef> for TypedSceneRef {
    type Error = SceneIdParseError;

    fn try_from(scene: &SceneRef) -> Result<Self, Self::Error> {
        match scene {
            SceneRef::Menu => Ok(Self::Menu),
            SceneRef::Loading => Ok(Self::Loading),
            SceneRef::Dialogue { id } => DialogueSceneId::parse(id)
                .map(Self::Dialogue)
                .ok_or_else(|| SceneIdParseError::UnknownDialogueId(id.clone())),
            SceneRef::Dilemma { id } => DilemmaSceneId::parse(id)
                .map(Self::Dilemma)
                .ok_or_else(|| SceneIdParseError::UnknownDilemmaId(id.clone())),
            SceneRef::Ending { id } => EndingSceneId::parse(id)
                .map(Self::Ending)
                .ok_or_else(|| SceneIdParseError::UnknownEndingId(id.clone())),
        }
    }
}

fn parse_path_with_outcome(id: &str, prefix: &str, min_stage: u8, max_stage: u8) -> Option<(u8, PathOutcomeId)> {
    let mut parts = id.split('.');
    if parts.next()? != prefix {
        return None;
    }

    let stage: u8 = parts.next()?.parse().ok()?;
    if stage < min_stage || stage > max_stage {
        return None;
    }

    let outcome = PathOutcomeId::parse(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }

    Some((stage, outcome))
}

fn parse_path_index(id: &str, prefix: &str, min_stage: u8, max_stage: u8) -> Option<u8> {
    let mut parts = id.split('.');
    if parts.next()? != prefix {
        return None;
    }

    let stage: u8 = parts.next()?.parse().ok()?;
    if stage < min_stage || stage > max_stage {
        return None;
    }

    if parts.next().is_some() {
        return None;
    }

    Some(stage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_static_dialogue_identifier() {
        assert_eq!(
            DialogueSceneId::parse("lab_1.a.pass"),
            Some(DialogueSceneId::Lab1aPass)
        );
    }

    #[test]
    fn parses_dynamic_dialogue_identifier() {
        assert_eq!(
            DialogueSceneId::parse("path_inaction.6.fail"),
            Some(DialogueSceneId::PathInaction {
                stage: 6,
                outcome: PathOutcomeId::Fail,
            })
        );
    }

    #[test]
    fn parses_dynamic_dilemma_identifier() {
        assert_eq!(
            DilemmaSceneId::parse("path_utilitarian.3"),
            Some(DilemmaSceneId::PathUtilitarian { stage: 3 })
        );
    }

    #[test]
    fn typed_scene_ref_rejects_unknown_id() {
        let scene = SceneRef::Dialogue {
            id: String::from("lab_999.invalid"),
        };

        assert!(matches!(
            TypedSceneRef::try_from(&scene),
            Err(SceneIdParseError::UnknownDialogueId(_))
        ));
    }
}
