use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainTypes {
    SteamTrain,
    PsychopathTruck,
}

impl TrainTypes {
    pub fn content(&self) -> &str {
        match self {
            TrainTypes::SteamTrain => include_str!("./steam_train.json"),
            TrainTypes::PsychopathTruck => include_str!("./psychopath_truck.json"),
        }
    }
}
