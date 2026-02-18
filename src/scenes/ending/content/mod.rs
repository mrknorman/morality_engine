use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
#[require(Transform, Visibility)]
pub enum EndingScene {
    IdioticPsychopath,
    ImpatientPsychopath,
    Leverophile,
    SelectiveDeontologist,
    TrueDeontologist,
    TrueNeutral,
}

impl EndingScene {
    pub fn content(&self) -> &str {
        match self {
            Self::IdioticPsychopath => include_str!("./lab/idiotic_psychopath.json"),
            Self::ImpatientPsychopath => include_str!("./lab/impatient_psychopath.json"),
            Self::Leverophile => include_str!("./lab/leverophile.json"),
            Self::SelectiveDeontologist => include_str!("./lab/selective_deontologist.json"),
            Self::TrueDeontologist => include_str!("./lab/true_deontologist.json"),
            Self::TrueNeutral => include_str!("./lab/true_neutral.json"),
        }
    }
}
