#[derive(Clone, Copy)]
pub enum TrainTypes {
    SteamTrain
}

impl TrainTypes{
    pub fn content(&self) -> &str {
        match self {
            TrainTypes::SteamTrain => include_str!("./steam_train.json")
        }
    }
}
