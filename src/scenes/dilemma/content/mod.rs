use bevy::prelude::*;

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
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DilemmaScene {
    Lab0(Lab0Dilemma),
    Lab1(Lab1Dilemma),
    PathInaction(DilemmaPathInaction, usize),
    PathPsychopath(DilemmaPathPsychopath, usize),
    Lab2(Lab2Dilemma),
    PathDeontological(DilemmaPathDeontological, usize),
    PathUtilitarian(DilemmaPathUtilitarian, usize),
    DayPersonal(DilemmaDayPersonal, usize),
    Lab3(Lab3Dilemma),
    Lab4(Lab4Dilemma),
}

impl DilemmaScene {
    pub fn content(&self) -> &'static str {
        match self {
            Self::Lab0(dilemma) => dilemma.content(),
            Self::Lab1(dilemma) => dilemma.content(),
            Self::PathInaction(dilemma, _) => dilemma.content(),
            Self::PathPsychopath(dilemma, _) => dilemma.content(),
            Self::Lab2(dilemma) => dilemma.content(),
            Self::PathDeontological(dilemma, _) => dilemma.content(),
            Self::PathUtilitarian(dilemma, _) => dilemma.content(),
            Self::DayPersonal(dilemma, _) => dilemma.content(),
            Self::Lab3(dilemma) => dilemma.content(),
            Self::Lab4(dilemma) => dilemma.content(),
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
    Lab3Dilemma {
        AsleepAtTheJob => "./lab/3/asleep_at_the_job.json"
    }
}

define_dilemma! {
    Lab4Dilemma {
        RandomDeaths => "./lab/4/random_deaths.json"
    }
}

define_dilemma! {
    DilemmaPathUtilitarian {
        OneFifth => "./lab/3/path_utilitarian/0/one_fifth.json",
        MarginOfError => "./lab/3/path_utilitarian/1/margin_of_error.json",
        NegligibleDifference => "./lab/3/path_utilitarian/2/negligible_difference.json",
        UnorthodoxSurgery => "./lab/3/path_utilitarian/3/unorthodox_surgery.json"
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

define_dilemma! {
    DilemmaPathPsychopath {
        TryAgain => "./lab/1/path_psychopath/try_again.json",
        OneOrTwo => "./lab/1/path_psychopath/one_or_two.json",
        DeathAtAConvent => "./lab/1/path_psychopath/death_at_a_convent.json",
        ProlongedSuffering => "./lab/1/path_psychopath/prolonged_suffering.json",
        TrainOfMassDestruction => "./lab/1/path_psychopath/train_of_mass_destruction.json"
    }
}

// Factory method for DilemmaScene to handle the usize parameter in Lab3PathInaction
impl DilemmaScene {
    pub const PATH_INACTION: [Self; 7] = [
        Self::PathInaction(DilemmaPathInaction::EmptyChoice, 0),
        Self::PathInaction(DilemmaPathInaction::PlentyOfTime, 1),
        Self::PathInaction(DilemmaPathInaction::LittleTime, 2),
        Self::PathInaction(DilemmaPathInaction::FiveOrNothing, 3),
        Self::PathInaction(DilemmaPathInaction::CancerCure, 4),
        Self::PathInaction(DilemmaPathInaction::OwnChild, 5),
        Self::PathInaction(DilemmaPathInaction::You, 6),
    ];

    pub const PATH_PSYCHOPATH: [Self; 5] = [
        Self::PathPsychopath(DilemmaPathPsychopath::TryAgain, 0),
        Self::PathPsychopath(DilemmaPathPsychopath::OneOrTwo, 1),
        Self::PathPsychopath(DilemmaPathPsychopath::DeathAtAConvent, 2),
        Self::PathPsychopath(DilemmaPathPsychopath::ProlongedSuffering, 3),
        Self::PathPsychopath(DilemmaPathPsychopath::TrainOfMassDestruction, 4),
    ];

    pub const PATH_DEONTOLOGICAL: [Self; 3] = [
        Self::PathDeontological(DilemmaPathDeontological::TrolleyerProblem, 0),
        Self::PathDeontological(DilemmaPathDeontological::TrolleyestProblem, 1),
        Self::PathDeontological(DilemmaPathDeontological::TrolleygeddonProblem, 2),
    ];

    pub const PATH_UTILITARIAN: [Self; 4] = [
        Self::PathUtilitarian(DilemmaPathUtilitarian::OneFifth, 0),
        Self::PathUtilitarian(DilemmaPathUtilitarian::MarginOfError, 1),
        Self::PathUtilitarian(DilemmaPathUtilitarian::NegligibleDifference, 2),
        Self::PathUtilitarian(DilemmaPathUtilitarian::UnorthodoxSurgery, 3),
    ];
}

define_dilemma! {
    DilemmaPathDeontological {
        TrolleyerProblem => "./lab/3/path_deontological/0/the_trolleyer_problem.json",
        TrolleyestProblem => "./lab/3/path_deontological/1/the_trolleyest_problem.json",
        TrolleygeddonProblem => "./lab/3/path_deontological/2/the_trolleygeddon_problem.json",
    }
}

define_dilemma! {
    DilemmaDayPersonal {
        GeorgeOrHannah => "./lab/day_1/0_george_or_hannah.json",
        TwistOfFate => "./lab/day_1/1_twist_of_fate.json",
        DecisionIgnition => "./lab/day_1/2_decision_ignition.json",
        AuthorizedWorkerVsPartyingYouths => "./lab/day_1/3_authorized_worker_vs_partying_youths.json",
        FatManOnBridge => "./lab/day_1/4_fat_man_on_bridge.json",
        FatConstructionWorkerVsPartyingYouths => "./lab/day_1/5_fat_construction_worker_vs_partying_youths.json",
        BoulderVsDeathCult => "./lab/day_1/6_boulder_vs_death_cult.json",
        CableCarMedalistVsSkiers => "./lab/day_1/7_cable_car_medalist_vs_skiers.json"
    }
}

impl DilemmaScene {
    pub const DAY_PERSONAL: [Self; 8] = [
        Self::DayPersonal(DilemmaDayPersonal::GeorgeOrHannah, 0),
        Self::DayPersonal(DilemmaDayPersonal::TwistOfFate, 1),
        Self::DayPersonal(DilemmaDayPersonal::DecisionIgnition, 2),
        Self::DayPersonal(DilemmaDayPersonal::AuthorizedWorkerVsPartyingYouths, 3),
        Self::DayPersonal(DilemmaDayPersonal::FatManOnBridge, 4),
        Self::DayPersonal(DilemmaDayPersonal::FatConstructionWorkerVsPartyingYouths, 5),
        Self::DayPersonal(DilemmaDayPersonal::BoulderVsDeathCult, 6),
        Self::DayPersonal(DilemmaDayPersonal::CableCarMedalistVsSkiers, 7),
    ];
}
