pub enum DialogueContent {
    Lab1(Lab1Dialogue),
    Lab2a(Lab2aDialogue),
    Lab2b(Lab2bDialogue),
    Lab3(Lab3Dialogue),
}

impl DialogueContent {
    pub fn content(&self) -> &'static str {
        match self {
            DialogueContent::Lab1(dialogue) => dialogue.content(),
            DialogueContent::Lab2a(dialogue) => dialogue.content(),
            DialogueContent::Lab2b(dialogue) => dialogue.content(),
            DialogueContent::Lab3(dialogue) => dialogue.content()

        }
    }
}

pub enum Lab1Dialogue {
    Intro,
    // Add more Lab 1 variants here if needed.
}

impl Lab1Dialogue {
    pub fn content(&self) -> &'static str {
        match self {
            Lab1Dialogue::Intro => include_str!("./lab/1/a/intro.json"),
        }
    }
}

pub enum Lab2aDialogue {
    Fail,
    PassIndecisive,
    Pass,
    PassSlow,
    FailVeryIndecisive,
}

impl Lab2aDialogue {
    pub fn content(&self) -> &'static str {
        match self {
            Lab2aDialogue::Fail => include_str!("./lab/2/a/fail.json"),
            Lab2aDialogue::PassIndecisive => include_str!("./lab/2/a/pass_indecisive.json"),
            Lab2aDialogue::FailVeryIndecisive => include_str!("./lab/2/a/fail_very_indecisive.json"),
            Lab2aDialogue::Pass => include_str!("./lab/2/a/pass.json"),
            Lab2aDialogue::PassSlow => include_str!("./lab/2/a/pass_slow.json"),
        }
    }
}

pub enum Lab2bDialogue {
    DilemmaIntro
}

impl Lab2bDialogue {
    pub fn content(&self) -> &'static str {
        match self {
            Lab2bDialogue::DilemmaIntro => include_str!("./lab/2/b/dilemma_intro.json")
        }
    }
}

pub enum Lab3Dialogue {
    FailIndecisive,
    PassSlowAgain,
    PassSlow,
    Pass,
    PathInaction,
}

impl Lab3Dialogue {
    pub fn content(&self) -> &'static str {
        match self {
            Lab3Dialogue::FailIndecisive => include_str!("./lab/3/a/fail_indecisive.json"),
            Lab3Dialogue::PassSlowAgain => include_str!("./lab/3/a/pass_slow_again.json"),
            Lab3Dialogue::PassSlow => include_str!("./lab/3/a/pass_slow.json"),
            Lab3Dialogue::Pass => include_str!("./lab/3/a/pass.json"),
            Lab3Dialogue::PathInaction => include_str!("./lab/3/a/pass_slow.json"),
        }
    }
}
