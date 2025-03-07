// Using the same define_dialogue macro for consistency
trait DilemmaProvider {
    fn content(&self) -> &'static str;
}

// Create a macro for generating dilemma enums (similar to define_dialogue)
macro_rules! define_dilemma {
    (
        $enum_name:ident {
            $($variant:ident => $path:literal),* $(,)?
        }
    ) => {
        pub enum $enum_name {
            $($variant),*
        }

        impl DilemmaProvider for $enum_name {
            fn content(&self) -> &'static str {
                match self {
                    $(Self::$variant => include_str!($path)),*
                }
            }
        }
    };
}

// Define the main dilemma content enum
pub enum DilemmaContent {
    Lab0(Lab0Dilemma),
    Lab1(Lab1Dilemma),
    PathInaction(DilemmaPathInaction, usize),
    Lab2(Lab2Dilemma)
}

impl DilemmaContent {
    pub fn content(&self) -> &'static str {
        match self {
            Self::Lab0(dilemma) => dilemma.content(),
            Self::Lab1(dilemma) => dilemma.content(),
            Self::PathInaction(dilemma, _) => dilemma.content(),
            Self::Lab2(dilemma) => dilemma.content()
        }
    }
}

// Define each dilemma type using the macro
define_dilemma! {
    Lab0Dilemma {
        IncompetentBandit => "./lab/0/incompetent_bandit.json",
    }
}

define_dilemma! {
    Lab1Dilemma {
        NearSightedBandit => "./lab/1/near_sighted_bandit.json",
    }
}

define_dilemma! {
    Lab2Dilemma {
        TheTrolleyProblem => "./lab/2/the_trolley_problem.json",
    }
}

define_dilemma! {
    DilemmaPathInaction {
        EmptyChoice => "./lab/2/path_inaction/0/empty_choice.json",
        PlentyOfTime => "./lab/2/path_inaction/1/plenty_of_time.json",
        LittleTime => "./lab/2/path_inaction/2/little_time.json",
        FiveOrNothing => "./lab/2/path_inaction/3/five_or_nothing.json",
        CancerCure => "./lab/2/path_inaction/4/a_cure_for_cancer.json",
        OwnChild => "./lab/2/path_inaction/5/your_own_child.json",
        You => "./lab/2/path_inaction/6/you.json",
    }
}

// Factory method for DilemmaContent to handle the usize parameter in Lab3PathInaction
impl DilemmaContent {
    pub fn path_inaction(stage: usize) -> Option<Self> {
        match stage {
            0 => Some(Self::PathInaction(DilemmaPathInaction::EmptyChoice, 0)),
            1 => Some(Self::PathInaction(DilemmaPathInaction::PlentyOfTime, 1)),
            2 => Some(Self::PathInaction(DilemmaPathInaction::LittleTime, 2)),
            3 => Some(Self::PathInaction(DilemmaPathInaction::FiveOrNothing, 3)),
            4 => Some(Self::PathInaction(DilemmaPathInaction::CancerCure, 4)),
            5 => Some(Self::PathInaction(DilemmaPathInaction::OwnChild, 5)),
            6 => Some(Self::PathInaction(DilemmaPathInaction::You, 6)),
            _ => None
        }
        
    }
}