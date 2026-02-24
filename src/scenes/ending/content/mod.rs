use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[require(Transform, Visibility)]
pub enum EndingScene {
    IdioticPsychopath,
    ImpatientPsychopath,
    Leverophile,
    ConfusedDeontologist,
    SelectiveDeontologist,
    TrueDeontologist,
    TrueNeutral,
    DayPersonalAllMenKilled,
    DayPersonalAllWomenKilled,
    DayPersonalIgnoredBomb,
    DayPersonalDidNothing,
}

impl EndingScene {
    pub fn content(&self) -> &str {
        match self {
            Self::IdioticPsychopath => include_str!("./lab/idiotic_psychopath.json"),
            Self::ImpatientPsychopath => include_str!("./lab/impatient_psychopath.json"),
            Self::Leverophile => include_str!("./lab/leverophile.json"),
            Self::ConfusedDeontologist => include_str!("./lab/confused_deontologist.json"),
            Self::SelectiveDeontologist => include_str!("./lab/selective_deontologist.json"),
            Self::TrueDeontologist => include_str!("./lab/true_deontologist.json"),
            Self::TrueNeutral => include_str!("./lab/true_neutral.json"),
            Self::DayPersonalAllMenKilled => include_str!("./lab/day_personal_all_men_killed.json"),
            Self::DayPersonalAllWomenKilled => {
                include_str!("./lab/day_personal_all_women_killed.json")
            }
            Self::DayPersonalIgnoredBomb => include_str!("./lab/day_personal_ignored_bomb.json"),
            Self::DayPersonalDidNothing => include_str!("./lab/day_personal_did_nothing.json"),
        }
    }
}
