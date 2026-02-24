use bevy::prelude::*;

trait DialogueProvider {
    fn content(&self) -> &'static str;
}

// Create a macro for generating dialogue enums
macro_rules! define_dialogue {
    (
        $enum_name:ident {
            $($variant:ident => $path:literal),* $(,)?
        }
    ) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum $enum_name {
            $($variant),*
        }

        impl DialogueProvider for $enum_name {
            fn content(&self) -> &'static str {
                match self {
                    $(Self::$variant => include_str!($path)),*
                }
            }
        }
    };
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[require(Transform, Visibility)]
pub enum DialogueScene {
    Lab0(Lab0Dialogue),
    Lab1a(Lab1aDialogue),
    Lab1b(Lab1bDialogue),
    Lab2a(Lab2aDialogue),
    Lab2b(Lab2bDialogue),
    Lab3a(Lab3aDialogue),
    Lab3b(Lab3bDialogue),
    Lab4(Lab4Dialogue),
}

impl DialogueScene {
    pub fn content(&self) -> &'static str {
        match self {
            Self::Lab0(dialogue) => dialogue.content(),
            Self::Lab1a(dialogue) => dialogue.content(),
            Self::Lab1b(dialogue) => dialogue.content(),
            Self::Lab2a(dialogue) => dialogue.content(),
            Self::Lab2b(dialogue) => dialogue.content(),
            Self::Lab3a(dialogue) => dialogue.content(),
            Self::Lab3b(dialogue) => dialogue.content(),
            Self::Lab4(dialogue) => dialogue.content(),
        }
    }

    pub fn path_inaction(number: usize, outcome: PathOutcome) -> Self {
        Self::Lab2a(Lab2aDialogue::PathInaction(InactionPath::new(
            number, outcome,
        )))
    }

    pub fn path_deontological(number: usize, outcome: PathOutcome) -> Self {
        Self::Lab3a(Lab3aDialogue::DeontologicalPath(DeontologicalPath::new(
            number, outcome,
        )))
    }

    pub fn path_utilitarian(number: usize, outcome: PathOutcome) -> Self {
        Self::Lab4(Lab4Dialogue::UtilitarianPath(UtilitarianPath::new(
            number, outcome,
        )))
    }
}

// Define each dialogue type using the macro
define_dialogue! {
    Lab0Dialogue {
        Intro => "./lab/0/intro.json",
    }
}

define_dialogue! {
    Lab1aDialogue {
        Fail => "./lab/1/a/fail.json",
        PassIndecisive => "./lab/1/a/pass_indecisive.json",
        FailVeryIndecisive => "./lab/1/a/fail_very_indecisive.json",
        Pass => "./lab/1/a/pass.json",
        PassSlow => "./lab/1/a/pass_slow.json",
    }
}

define_dialogue! {
    Lab1bDialogue {
        DilemmaIntro => "./lab/1/b/intro.json",
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lab2aDialogue {
    FailIndecisive,
    Fail,
    PassSlowAgain,
    PassSlow,
    Pass,
    PathInaction(InactionPath),
}

define_dialogue! {
    Lab2bDialogue {
        Intro => "./lab/2/b/intro.json",
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
// Define outcome enum for more flexibility
pub enum PathOutcome {
    Pass,
    Fail,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
// Path configuration for the inaction path
pub struct InactionPath {
    number: usize,
    outcome: PathOutcome,
}

impl InactionPath {
    pub fn new(number: usize, outcome: PathOutcome) -> Self {
        let normalized_number = number.min(6);
        if normalized_number != number {
            warn!("inaction path index {number} is out of range; clamped to {normalized_number}");
        }
        Self {
            number: normalized_number,
            outcome,
        }
    }

    // Helper to get the JSON content based on path parameters
    fn get_json_content(&self) -> &'static str {
        match (&self.outcome, self.number) {
            // All Pass outcomes point to path 7/pass.json
            (PathOutcome::Pass, _) => include_str!("./lab/2/path_inaction/6/pass.json"),

            // Fail outcomes go to their respective path number
            (PathOutcome::Fail, 0) => include_str!("./lab/2/path_inaction/0/fail.json"),
            (PathOutcome::Fail, 1) => include_str!("./lab/2/path_inaction/1/fail.json"),
            (PathOutcome::Fail, 2) => include_str!("./lab/2/path_inaction/2/fail.json"),
            (PathOutcome::Fail, 3) => include_str!("./lab/2/path_inaction/3/fail.json"),
            (PathOutcome::Fail, 4) => include_str!("./lab/2/path_inaction/4/fail.json"),
            (PathOutcome::Fail, 5) => include_str!("./lab/2/path_inaction/5/fail.json"),
            (PathOutcome::Fail, 6) => include_str!("./lab/2/path_inaction/6/fail.json"),

            _ => unreachable!("Invalid path configuration"),
        }
    }
}

// Implement DialogueProvider for Lab3Dialogue
impl DialogueProvider for Lab2aDialogue {
    fn content(&self) -> &'static str {
        match self {
            Self::FailIndecisive => include_str!("./lab/2/a/fail_indecisive.json"),
            Self::Fail => include_str!("./lab/2/a/fail.json"),
            Self::PassSlowAgain => include_str!("./lab/2/a/pass_slow_again.json"),
            Self::PassSlow => include_str!("./lab/2/a/pass_slow.json"),
            Self::Pass => include_str!("./lab/2/a/pass.json"),
            Self::PathInaction(path) => path.get_json_content(),
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]

pub struct DeontologicalPath {
    number: usize,
    outcome: PathOutcome,
}

impl DeontologicalPath {
    pub fn new(number: usize, outcome: PathOutcome) -> Self {
        let normalized_number = number.min(3);
        if normalized_number != number {
            warn!(
                "deontological path index {number} is out of range; clamped to {normalized_number}"
            );
        }
        Self {
            number: normalized_number,
            outcome,
        }
    }

    // Helper to get the JSON content based on path parameters
    fn get_json_content(&self) -> &'static str {
        match (&self.outcome, self.number) {
            // All Pass outcomes point to path 7/pass.json
            (PathOutcome::Pass, _) => include_str!("./lab/3/path_deontological/pass.json"),

            // Fail outcomes go to their respective path number
            (PathOutcome::Fail, 0) => include_str!("./lab/3/path_deontological/0/fail.json"),
            (PathOutcome::Fail, 1) => include_str!("./lab/3/path_deontological/1/fail.json"),
            (PathOutcome::Fail, 2) => include_str!("./lab/3/path_deontological/2/fail.json"),
            (PathOutcome::Fail, 3) => include_str!("./lab/3/path_deontological/3/fail.json"),

            _ => unreachable!("Invalid path configuration"),
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Lab3aDialogue {
    FailIndecisive,
    FailInaction,
    PassUtilitarian,
    DeontologicalPath(DeontologicalPath),
}

impl DialogueProvider for Lab3aDialogue {
    fn content(&self) -> &'static str {
        match self {
            Self::PassUtilitarian => include_str!("./lab/3/a/pass_utilitarian.json"),
            Self::FailIndecisive => include_str!("./lab/3/a/fail_indecisive.json"),
            Self::FailInaction => include_str!("./lab/3/a/fail_inaction.json"),
            Self::DeontologicalPath(path) => path.get_json_content(),
        }
    }
}

define_dialogue! {
    Lab3bDialogue {
        Intro => "./lab/3/b/intro.json",
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Lab4Dialogue {
    UtilitarianPath(UtilitarianPath),
    Outro,
}

impl DialogueProvider for Lab4Dialogue {
    fn content(&self) -> &'static str {
        match self {
            Self::UtilitarianPath(path) => path.get_json_content(),
            Self::Outro => include_str!("./lab/4/outro.json"),
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]

pub struct UtilitarianPath {
    number: usize,
    outcome: PathOutcome,
}

impl UtilitarianPath {
    pub fn new(number: usize, outcome: PathOutcome) -> Self {
        let normalized_number = number.clamp(1, 4);
        if normalized_number != number {
            warn!("utilitarian path index {number} is out of range; clamped to {normalized_number}");
        }
        Self {
            number: normalized_number,
            outcome,
        }
    }

    // Helper to get the JSON content based on path parameters
    fn get_json_content(&self) -> &'static str {
        match (&self.outcome, self.number) {
            (PathOutcome::Pass, 1) => include_str!("./lab/4/path_utilitarian/1/pass.json"),
            (PathOutcome::Fail, 1) => include_str!("./lab/4/path_utilitarian/1/fail.json"),

            (PathOutcome::Pass, 2) => include_str!("./lab/4/path_utilitarian/2/pass.json"),
            (PathOutcome::Fail, 2) => include_str!("./lab/4/path_utilitarian/2/fail.json"),

            (PathOutcome::Pass, 3) => include_str!("./lab/4/path_utilitarian/3/pass.json"),
            (PathOutcome::Fail, 3) => include_str!("./lab/4/path_utilitarian/3/fail.json"),

            (PathOutcome::Pass, 4) => include_str!("./lab/4/path_utilitarian/4/pass.json"),
            (PathOutcome::Fail, 4) => include_str!("./lab/4/path_utilitarian/4/fail.json"),

            _ => unreachable!("Invalid path configuration"),
        }
    }
}
