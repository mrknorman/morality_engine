pub enum DilemmaContent {
    Lab1(Lab1Dilemma),
    Lab2(Lab2Dilemma)
}

pub enum Lab1Dilemma {
    IncompetentBandit,
}

impl Lab1Dilemma {
    pub fn content(&self) -> &'static str {
        match self {
            Lab1Dilemma::IncompetentBandit => include_str!("./lab/1/incompetent_bandit.json"),
        }
    }
}

pub enum Lab2Dilemma {
    NearSightedBandit,
}

impl Lab2Dilemma {
    pub fn content(&self) -> &'static str {
        match self {
            Lab2Dilemma::NearSightedBandit => include_str!("./lab/2/near_sighted_bandit.json"),
        }
    }
}

impl DilemmaContent {
    pub fn content(&self) -> &'static str {
        match self {
            DilemmaContent::Lab1(dilemma) => dilemma.content(),
            DilemmaContent::Lab2(dilemma) => dilemma.content()
        }
    }
}

