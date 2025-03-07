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

// Define the main dialogue content enum
pub enum DialogueContent {
    Lab0(Lab0Dialogue),
    Lab1a(Lab1aDialogue),
    Lab1b(Lab1bDialogue),
    Lab2a(Lab2aDialogue),
    Lab2b(Lab2bDialogue),
    Lab3a(Lab3aDialogue),
    Lab3b(Lab3bDialogue)

}

impl DialogueContent {
    pub fn content(&self) -> &'static str {
        match self {
            Self::Lab0(dialogue) => dialogue.content(),
            Self::Lab1a(dialogue) => dialogue.content(),
            Self::Lab1b(dialogue) => dialogue.content(),
            Self::Lab2a(dialogue) => dialogue.content(),
            Self::Lab2b(dialogue) => dialogue.content(),
            Self::Lab3a(dialogue) => dialogue.content(),
            Self::Lab3b(dialogue) => dialogue.content(),
        }
    }

    pub fn path_inaction(number: usize, outcome: PathOutcome) -> Self {
        Self::Lab2a(Lab2aDialogue::PathInaction(InactionPath::new(number, outcome)))
    }

    pub fn path_deontological(number: usize, outcome: PathOutcome) -> Self {
        Self::Lab3a(Lab3aDialogue::DeontologicalPath(DeontologicalPath::new(number, outcome)))
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

// Define outcome enum for more flexibility
pub enum PathOutcome {
    Pass,
    Fail
}

// Path configuration for the inaction path
pub struct InactionPath {
    number: usize,
    outcome: PathOutcome,
}

impl InactionPath {
    pub fn new(number: usize, outcome: PathOutcome) -> Self {
        assert!(number <= 6, "Path number must be less than 6");
        Self { number, outcome }
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

pub struct DeontologicalPath {
    number: usize,
    outcome: PathOutcome,
}

impl DeontologicalPath {
    pub fn new(number: usize, outcome: PathOutcome) -> Self {
        assert!(number <= 4, "Path number must be less than 4");
        Self { number, outcome }
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

pub enum Lab3aDialogue {
    Fail,
    Pass,
    DeontologicalPath(DeontologicalPath)
}

impl DialogueProvider for Lab3aDialogue {
    fn content(&self) -> &'static str {
        match self {
            Self::Pass => include_str!("./lab/3/a/pass.json"),
            Self::Fail => include_str!("./lab/3/a/fail.json"),
            Self::DeontologicalPath(path) => path.get_json_content(),
        }
    }
}

define_dialogue! {
    Lab3bDialogue {
        Intro => "./lab/3/b/intro.json",
    }
}